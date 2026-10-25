use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

// Client for the ZK timeclock pull protocol over TCP 4370.
// Dependency-free, blocking, one TCP session per call.
//
// The packet on the wire is an 8-byte TCP top (marker 0x5050, 0x7D82, then
// a u32 little-endian total length) plus an 8-byte header (command, checksum,
// session id, reply id) plus payload. Bulk reads use command 1503 with an
// inner table byte (13 = attendance log), then fixed 1024-byte chunks via
// command 1504 whose payloads arrive tagged 2002.
const PREP_1: u16 = 0x5050;
const PREP_2: u16 = 0x7D82; // on the wire, not the 0x7282 in pyzk's constants
const USHRT_MAX: u32 = 65535;

const CMD_CONNECT: u16 = 1000;
const CMD_EXIT: u16 = 1001;
const CMD_RESTART: u16 = 1004;
const CMD_FREE_SIZES: u16 = 50;
const CMD_RWB: u16 = 1503;
const CMD_READ_CHUNK: u16 = 1504;
const CMD_DATA_WRRQ: u16 = 1502;

const CMD_ACK_OK: u16 = 2000;
const CMD_DATA: u16 = 1501;
const CMD_ACK_DATA: u16 = 2002;

// Live capture: the device pushes each punch at us instead of waiting to be
// polled. Registration is a one-shot command; after it the socket carries
// unsolicited event frames until we disconnect.
const CMD_REG_EVENT: u16 = 500;
const CMD_CANCEL_CAPTURE: u16 = 62;
const CMD_ENABLE_DEVICE: u16 = 1002;
const CMD_SET_TIME: u16 = 202;
const CMD_START_VERIFY: u16 = 60;

// Event mask. EF_ATTLOG (1) = real-time attendance verification. The other bits
// are door control, user management and system events, which we have no use for.
const EF_ATTLOG: u32 = 1;

// How long a live socket blocks before the capture loop re-checks for a stop
// request.
//
// The device sends nothing while idle, and that silence is normal, so the read
// has to time out for the loop to get control back. This value is therefore the
// shutdown latency: a process asked to stop waits this long before it can drop
// the device socket. Keep it short enough that a restart is quick, long enough
// that a busy device is not read in a tight loop.
const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(500);

const CHUNK: usize = 1024;

// Ceiling for the goodbye handshake in Drop. A blocking read that runs the full
// session timeout would stall a worker thread for three minutes per failure.
const DROP_TIMEOUT: Duration = Duration::from_secs(5);

// How long one frame read may keep retrying past a socket read timeout before
// the pull is called off. Sized to match the caller's session timeout so a
// genuinely stalled device still fails in bounded time.
const READ_BUDGET: Duration = Duration::from_secs(180);

// The `1503` size-probe payload: pack("<bhii", 1, 13, 0, 0) = a table byte, the
// inner command as a LITTLE-ENDIAN U16, and two zero words. Encoding the inner
// command as a bare u8 is one byte short, and the device then answers the
// SECOND probe with 2001, so the wrong width fails late and looks like a device
// problem rather than a framing one.
const RWB_PRIME: [u8; 11] = [1, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0];

// One attendance record: uid u16, user id 24 bytes, status u8,
// timestamp u32, punch u8, 8 reserved. Other firmwares use 8 or 16 byte
// layouts, so callers derive the width from the advertised record count.
pub const RECORD_40: usize = 40;

#[derive(Debug, Clone)]
pub struct AttendanceRow {
    pub user_id: String,
    pub device_time: u32,
    pub status: u8,
    pub punch: u8,
}

// Word-sum complement over command, checksum slot, session, reply id, payload.
// Signed arithmetic on purpose, mirroring the reference: reduce modulo 65535
// by subtraction, bitwise-not, then add 65535 back while negative. A u16-only
// version is off by one (0xFC18 vs the wire-captured 0xFC17).
fn checksum(words: &[u8]) -> u16 {
    let mut total: i32 = 0;
    let mut i = 0;
    while i + 1 < words.len() {
        total += u16::from_le_bytes([words[i], words[i + 1]]) as i32;
        i += 2;
    }
    if i < words.len() {
        total += words[i] as i32;
    }
    while total > USHRT_MAX as i32 {
        total -= USHRT_MAX as i32;
    }
    total = !total;
    while total < 0 {
        total += USHRT_MAX as i32;
    }
    total as u16
}

pub struct ZkPull {
    stream: TcpStream,
    session: u16,
    reply: u16,
    // True once the session was closed politely, so Drop stays quiet.
    closed: bool,
}

impl ZkPull {
    pub fn connect(addr: &SocketAddr, timeout: Duration) -> Result<Self> {
        let stream = TcpStream::connect_timeout(addr, timeout)?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        let mut client = Self {
            stream,
            session: 0,
            reply: (USHRT_MAX - 1) as u16,
            closed: false,
        };
        let (code, session, reply, _) = client.round_trip(CMD_CONNECT, &[], 0)?;
        // Only a real handshake proves pull support. A bare TCP accept means
        // nothing: this device family resets on a wrong framing word, and
        // some firmwares answer 0x07f0 to refuse pull outright.
        if code != CMD_ACK_OK {
            // No session was ever granted, so there is nothing to close. Without
            // this the Drop below would send CMD_EXIT on a session we never got,
            // which the device reads as a bogus frame on its idle listener.
            client.closed = true;
            return Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!("device refused pull handshake, answered {code:#06x}"),
            ));
        }
        client.session = session;
        client.reply = reply;
        Ok(client)
    }

    /// Build a frame that answers one specific frame, echoing its reply id.
    ///
    /// `frame` increments the id for a request/response exchange, which is right
    /// for commands we initiate. An event ACK is different: the device already
    /// picked an id, and answering with a different one leaves the punch
    /// unacknowledged, so the device sends it again.
    fn frame_with_reply(&self, command: u16, payload: &[u8], reply: u16) -> Vec<u8> {
        // Same layout as `frame`: the checksum is taken over a head whose reply
        // field is zero, and the reply field that is actually sent is the last
        // u16 of the inner block. Only the increment is missing here.
        let mut head = Vec::with_capacity(8 + payload.len());
        head.extend_from_slice(&command.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&self.session.to_le_bytes());
        head.extend_from_slice(&reply.to_le_bytes());
        head.extend_from_slice(payload);
        let sum = checksum(&head);
        let mut inner = Vec::with_capacity(8 + payload.len());
        inner.extend_from_slice(&command.to_le_bytes());
        inner.extend_from_slice(&sum.to_le_bytes());
        inner.extend_from_slice(&self.session.to_le_bytes());
        inner.extend_from_slice(&reply.to_le_bytes());
        inner.extend_from_slice(payload);
        let mut out = Vec::with_capacity(8 + inner.len());
        out.extend_from_slice(&PREP_1.to_le_bytes());
        out.extend_from_slice(&PREP_2.to_le_bytes());
        out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
        out.extend_from_slice(&inner);
        out
    }

    fn frame(&self, command: u16, payload: &[u8]) -> Vec<u8> {
        let mut head = Vec::with_capacity(8 + payload.len());
        head.extend_from_slice(&command.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&self.session.to_le_bytes());
        head.extend_from_slice(&self.reply.to_le_bytes());
        head.extend_from_slice(payload);
        let sum = checksum(&head);
        let next = if self.reply as u32 + 1 >= USHRT_MAX {
            (self.reply as u32 + 1 - USHRT_MAX) as u16
        } else {
            self.reply + 1
        };
        let mut inner = Vec::with_capacity(8 + payload.len());
        inner.extend_from_slice(&command.to_le_bytes());
        inner.extend_from_slice(&sum.to_le_bytes());
        inner.extend_from_slice(&self.session.to_le_bytes());
        inner.extend_from_slice(&next.to_le_bytes());
        inner.extend_from_slice(payload);
        let mut out = Vec::with_capacity(8 + inner.len());
        out.extend_from_slice(&PREP_1.to_le_bytes());
        out.extend_from_slice(&PREP_2.to_le_bytes());
        out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
        out.extend_from_slice(&inner);
        out
    }

    // Send one command and read its single reply.
    //
    // Mirrors the reference client: one send, then drain until the TCP top's
    // length field is satisfied. The length field names the whole reply minus
    // the 8 top bytes, so a complete frame is `length + 8` bytes and its payload
    // is everything after the 8-byte top and 8-byte reply header.
    //
    // Returns the payload AND its length. A caller that re-reads the socket
    // instead consumes the NEXT frame, because this one has already left it.
    fn round_trip(
        &mut self,
        command: u16,
        payload: &[u8],
        window: usize,
    ) -> Result<(u16, u16, u16, Vec<u8>)> {
        self.stream.write_all(&self.frame(command, payload))?;
        // One read at window size, then top up if the frame is bigger. Reading
        // only once would leave the tail in the socket and shift every field.
        let mut buf = vec![0u8; window + 8];
        let mut n_read = self.stream.read(&mut buf)?;
        if n_read == 0 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "device closed connection",
            ));
        }
        let want = if n_read >= 8 {
            u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize + 8
        } else {
            window + 8
        };
        if want > buf.len() {
            buf.resize(want, 0);
        }
        while n_read < want {
            let n = self.stream.read(&mut buf[n_read..want])?;
            if n == 0 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("reply stopped at {n_read} of {want} bytes"),
                ));
            }
            n_read += n;
        }
        if want < 16 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("reply of {want} bytes has no header"),
            ));
        }
        let code = u16::from_le_bytes([buf[8], buf[9]]);
        let session = u16::from_le_bytes([buf[12], buf[13]]);
        let reply = u16::from_le_bytes([buf[14], buf[15]]);
        self.reply = reply;
        // Slice to `want`, never to the buffer end: a resize zero-fills the
        // slack, and handing that back shifts every parsed field.
        let body = buf[16..want].to_vec();
        Ok((code, session, reply, body))
    }

    // Read exactly `count` bytes, tolerating a socket read timeout mid-frame.
    //
    // A timeout on a partially filled buffer surfaces as EAGAIN, not as a clean
    // timeout, so a bare `read_exact` fails the whole pull the first time the
    // device splits a frame across two TCP segments. Keep reading on
    // WouldBlock/TimedOut until the count is met or the budget runs out.
    fn read_exactly(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        let mut got = 0;
        let deadline = Instant::now() + READ_BUDGET;
        while got < count {
            match self.stream.read(&mut buf[got..]) {
                Ok(0) => {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "device closed connection",
                    ));
                }
                Ok(n) => got += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        ErrorKind::WouldBlock | ErrorKind::TimedOut
                    ) =>
                {
                    if Instant::now() >= deadline {
                        return Err(Error::new(
                            ErrorKind::TimedOut,
                            format!("timed out with {got} of {count} bytes"),
                        ));
                    }
                    // Bytes may already be buffered; yield briefly and retry.
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => return Err(e),
            }
        }
        Ok(buf)
    }

    /// Like `read_exactly`, but gives up as soon as `running` goes false.
    ///
    /// The plain version retries for the whole read budget, which is generous on
    /// purpose because table pulls legitimately need it. The live reader cannot
    /// afford that: `spawn_blocking` cannot be interrupted, so a cancel would not
    /// reach the thread until the budget expired, `CMD_EXIT` would go out late or
    /// not at all, and the device would keep its session slot and sit there frozen
    /// until it timed the client out on its own.
    fn read_exactly_cancellable(
        &mut self,
        count: usize,
        running: &mut impl FnMut() -> bool,
    ) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        let mut got = 0;
        let deadline = Instant::now() + READ_BUDGET;
        while got < count {
            if !running() {
                return Err(Error::new(ErrorKind::Interrupted, "cancelled mid-frame"));
            }
            match self.stream.read(&mut buf[got..]) {
                Ok(0) => {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "device closed connection",
                    ));
                }
                Ok(n) => got += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        ErrorKind::WouldBlock | ErrorKind::TimedOut
                    ) =>
                {
                    if Instant::now() >= deadline {
                        return Err(Error::new(
                            ErrorKind::TimedOut,
                            format!("timed out with {got} of {count} bytes"),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => return Err(e),
            }
        }
        Ok(buf)
    }

    // Read one reply head: 8-byte TCP top (marker + total length) then the
    // 8-byte reply header (command, checksum, session, reply id).
    fn read_head(&mut self) -> Result<(u16, usize)> {
        let top = self.read_exactly(8)?;
        // The total doubles as validation: a garbage frame names an absurd size.
        let total = u32::from_le_bytes([top[4], top[5], top[6], top[7]]) as usize;
        if total > 1 << 24 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("implausible reply length {total}"),
            ));
        }
        let head = self.read_exactly(8)?;
        let code = u16::from_le_bytes([head[0], head[1]]);
        let reply = u16::from_le_bytes([head[6], head[7]]);
        // The device's reply counter is echoed back per response header. Every
        // reader must update it, or the next frame carries a stale id and the
        // device answers the follow-up with an error instead of data.
        self.reply = reply;
        Ok((code, total))
    }

    fn free_sizes(&mut self) -> Result<[i32; 20]> {
        log::debug!("zkpull asking free sizes");
        self.stream.write_all(&self.frame(CMD_FREE_SIZES, &[])).map_err(|e| {
            Error::new(ErrorKind::Other, format!("free_sizes send: {e}"))
        })?;
        // One reply: an 8-byte header plus twenty little-endian i32 tallies.
        // The tallies ride on the ACK frame itself (total 88), not on a
        // following data reply. Reading past the head would eat the next
        // frame's marker as if it were payload.
        let (code, total) = self
            .read_head()
            .map_err(|e| Error::new(ErrorKind::Other, format!("free_sizes head: {e}")))?;
        if code != CMD_ACK_OK && code != CMD_DATA && code != CMD_ACK_DATA {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("free sizes answered {code}"),
            ));
        }
        if total != 88 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("free sizes carry {total} bytes, expected 88"),
            ));
        }
        let body = self.read_exactly(80)?;
        let mut fields = [0i32; 20];
        for (i, slot) in fields.iter_mut().enumerate() {
            *slot = i32::from_le_bytes([
                body[i * 4],
                body[i * 4 + 1],
                body[i * 4 + 2],
                body[i * 4 + 3],
            ]);
        }
        Ok(fields)
    }

    fn payload_size(&mut self) -> Result<(usize, Vec<u8>)> {
        log::debug!("zkpull asking for size");
        // One round trip, like the reference client: send 1503 once and read that one
        // reply. Sending it twice left the first reply in the socket and shifted
        // every later read by one frame.
        //
        // The reference client's logic: if the reply is a 1501 DATA frame, the
        // payload IS the table, and anything the first read missed arrives as a
        // raw continuation with no further top or header. If it is a 2000 ACK,
        // the size lives in bytes 1..5 and the table comes in aligned chunks.
        // Either way the size is a byte count, not a field to trust.
        let (code, _session, _reply, body) = self.round_trip(CMD_RWB, &RWB_PRIME, CHUNK)?;
        if code != CMD_DATA && code != CMD_ACK_OK {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("size probe answered {code}"),
            ));
        }
        if code == CMD_DATA {
            return Ok((body.len(), body));
        }
        // ACK_OK: the size field is in the payload, and the table follows in chunks.
        if body.len() < 5 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("size header is {} bytes", body.len()),
            ));
        }
        let size = u32::from_le_bytes([body[1], body[2], body[3], body[4]]) as usize;
        Ok((size, Vec::new()))
    }

    fn chunk(&mut self, start: u32, size: u32) -> Result<Vec<u8>> {
        log::debug!("zkpull chunk @{start} size {size}");
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&start.to_le_bytes());
        payload.extend_from_slice(&size.to_le_bytes());
        self.stream
            .write_all(&self.frame(CMD_READ_CHUNK, &payload))?;
        // A chunk reply carries the data bytes inline, tagged 1501 (the reference
        // implementation expects 2002 and over-reads by 32 bytes, which is how
        // it loses chunks on this firmware). Read exactly `size` and no more:
        // over-reading here would steal the head of the NEXT reply.
        let (code, _total) = self.read_head()?;
        log::info!("zkpull chunk @{start} want {size} answered {code}");
        if code != CMD_DATA && code != CMD_ACK_DATA {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("chunk @{start} answered {code}"),
            ));
        }
        // Reuse the bounded reader so a stalled device cannot pin this thread.
        let out = self.read_exactly(size as usize)?;
        Ok(out)
    }

    fn finish_read(&mut self) {
        let _ = self.round_trip(CMD_DATA_WRRQ, &[], 0);
        let _ = self.round_trip(CMD_EXIT, &[], 0);
        self.closed = true;
    }

    /// Ask the device to restart its network stack.
    ///
    /// Deliberately not exposed anywhere. It does NOT reboot the machine: the
    /// X105 reboots only from its own menu button or a power cycle, and this
    /// command leaves it powered and logged in. A UI button for it would look
    /// like a fix for the stuck-session state and quietly fail to clear it.
    ///
    /// Kept because it is part of the protocol, not as a recovery tool. The
    /// device drops the connection without answering, so a successful send is
    /// the strongest signal available.
    pub fn restart(&mut self) -> Result<()> {
        self.stream.write_all(&self.frame(CMD_RESTART, &[]))?;
        let _ = self.stream.flush();
        Ok(())
    }

    /// Connect, send a restart, and return whether the command was delivered.
    pub fn restart_device(addr: &SocketAddr, timeout: Duration) -> Result<()> {
        let mut client = Self::connect(addr, timeout)?;
        let result = client.restart();
        let _ = client.stream.shutdown(std::net::Shutdown::Both);
        result
    }

    /// Encode a local wall-clock datetime into the device's packed time word.
    /// Port of pyzk's __encode_time: year*12*31 + month*31 + day, scaled by
    /// seconds-per-day, plus the time-of-day in seconds.
    pub fn encode_device_time(dt: &chrono::NaiveDateTime) -> u32 {
        use chrono::{Datelike, Timelike};
        (((dt.year() as i64 % 100) * 12 * 31 + ((dt.month() as i64 - 1) * 31) + (dt.day() as i64 - 1))
            * (24 * 60 * 60)
            + ((dt.hour() as i64 * 60 + dt.minute() as i64) * 60 + dt.second() as i64)) as u32
    }

    pub fn set_time(&mut self, timestamp: u32) -> Result<()> {
        let (code, ..) = self.round_trip(CMD_SET_TIME, &timestamp.to_le_bytes(), 0)?;
        if code != CMD_ACK_OK {
            return Err(Error::new(
                ErrorKind::Other,
                format!("set_time rejected: code {code}"),
            ));
        }
        Ok(())
    }

    /// Connect, set the device clock, and disconnect.
    pub fn set_device_time(addr: &SocketAddr, timeout: Duration, timestamp: u32) -> Result<()> {
        let mut client = Self::connect(addr, timeout)?;
        let result = client.set_time(timestamp);
        let _ = client.stream.shutdown(std::net::Shutdown::Both);
        result
    }

    // Pull the whole attendance table into memory in fixed chunks.
    pub fn pull_attendance(&mut self) -> Result<(Vec<u8>, i32)> {
        let fields = self
            .free_sizes()
            .map_err(|e| Error::new(ErrorKind::Other, format!("stage=free_sizes {e}")))?;
        let records = fields[8];
        let (total, inline) = self
            .payload_size()
            .map_err(|e| Error::new(ErrorKind::Other, format!("stage=payload_size {e}")))?;
        let mut raw = Vec::with_capacity(total);
        let mut start = 0u32;
        // A 1501 probe reply already carries the whole opening of the table, so
        // there is nothing left to request: the reference client returns that
        // payload as-is instead of issuing further 1504 chunks. Continuing from
        // the inline length asked the device for a non-FFC0 offset and it
        // answered 2001 (invalid).
        if !inline.is_empty() {
            // The device answers the probe with a DATA frame when the table fits
            // in one buffer, and that payload is the whole table. If it does not
            // fit, this client cannot continue: a 1504 chunk request is only
            // valid on an FFC0-aligned offset, and the probe's inline length is
            // not one. Fail loudly rather than store a silently short table.
            if inline.len() < total {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "device sent {} bytes but announced {total}; \
                         a multi-buffer table needs an FFC0-aligned chunk read",
                        inline.len()
                    ),
                ));
            }
            self.finish_read();
            return Ok((inline, records));
        }
        while (start as usize) < total {
            let want = ((total - start as usize).min(CHUNK)) as u32;
            let got = self.chunk(start, want).map_err(|e| {
                Error::new(ErrorKind::Other, format!("stage=chunk@{start} {e}"))
            })?;
            if got.is_empty() {
                break;
            }
            raw.extend_from_slice(&got);
            start += got.len() as u32;
        }
        self.finish_read();
        Ok((raw, records))
    }

    /// Subscribe to real-time events and hand each punch to `on_punch`.
    ///
    /// The device pushes frames after registration without being asked, so this
    /// owns the socket until the caller's shutdown signal arrives. It registers
    /// for attendance only, streams, and returns when `running` goes false or the
    /// socket dies.
    pub fn live_capture(
        &mut self,
        mut running: impl FnMut() -> bool,
        mut on_punch: impl FnMut(AttendanceRow),
    ) -> Result<u64> {
        // Put the device into a known state before registering, in the order the
        // reference client uses: cancel any pending capture, ensure it is waiting
        // for an order, ensure it is enabled. Skipping the middle step leaves the
        // device in enrollment or verification state and it pushes nothing.
        self.round_trip(CMD_CANCEL_CAPTURE, &[], 0)?;
        self.round_trip(CMD_START_VERIFY, &[], 0)?;
        self.round_trip(CMD_ENABLE_DEVICE, &[], 0)?;
        let (code, _session, reply, _) = self.round_trip(CMD_REG_EVENT, &EF_ATTLOG.to_le_bytes(), 0)?;
        if code != CMD_ACK_OK {
            return Err(Error::new(
                ErrorKind::Other,
                format!("reg_event rejected: code {code}"),
            ));
        }
        self.reply = reply;
        // Fall through to the event loop below.

        let mut delivered = 0u64;
        // Read timeout is a heartbeat window, not a failure: a device with no
        // punches sends nothing, and that is normal, not an error.
        //
        // It is also the granularity at which a stop signal is noticed: the loop
        // can only check `running()` between reads, so a long timeout means a
        // shutdown waits that long before the socket is released. A quiet device
        // and a cancelled session read the same way, so poll finely instead.
        self.stream
            .set_read_timeout(Some(EVENT_POLL_INTERVAL))?;
        log::debug!("live_capture: registered, awaiting events");
        loop {
            if !running() {
                log::debug!("live_capture: stop signal received, ending session");
                break;
            }
            // Acknowledge the frame before deciding what it was. The reference
            // client ACKs every frame it receives, including ones it discards, and
            // a device that goes unacknowledged stops pushing.
            let frame = match self.read_event_frame(&mut running) {
                Ok(frame) => frame,
                // A cancelled read is the normal way this loop ends, so it falls
                // through to the release below rather than bubbling up as a fault.
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    log::debug!("live_capture: cancelled mid-frame, ending session");
                    break;
                }
                Err(e)
                    if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
                {
                    continue
                }
                Err(e) => return Err(e),
            };
            // The ACK has to carry the SAME reply id as the frame it answers:
            // `decode_event` copies the device's reply id into `self.reply`, and
            // `frame` would bump it by one. Sending the bumped value made the
            // device treat the ACK as unacknowledged and re-send the same punch
            // forever.
            let ack_reply = u16::from_le_bytes([frame[14], frame[15]]);
            let ack = self.frame_with_reply(CMD_ACK_OK, &[], ack_reply);
            self.stream.write_all(&ack)?;
            match self.decode_event(&frame) {
                Ok(Some(row)) => {
                    delivered += 1;
                    log::debug!("live_capture: punch from uid {}", row.user_id);
                    on_punch(row);
                }
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        }
        // Release the registration so the device stops pushing into a socket we
        // are about to close.
        let _ = self.round_trip(CMD_CANCEL_CAPTURE, &[], 0);
        Ok(delivered)
    }

    /// Read one event frame, returning a row when it carries an attendance punch.
    ///
    /// `Ok(None)` means a well-formed frame we have no use for (a door or system
    /// event), so the caller loops. A short read is not an error: the frame is
    /// announced in the top, so read exactly that much.
    fn read_event_frame(&mut self, running: &mut impl FnMut() -> bool) -> Result<Vec<u8>> {
        // The device writes a whole frame per event, so the top and the header
        // arrive together. Read 16 bytes up front, exactly as the reference
        // client does, rather than reading a top and then a header as if they
        // were separate.
        let mut frame = self.read_exactly_cancellable(16, running)?;
        let total = u32::from_le_bytes([frame[4], frame[5], frame[6], frame[7]]) as usize;
        // `total` names the header plus the payload, so the payload length is
        // `total - 8`. A frame shorter than its own header means the stream is
        // desynchronised, and guessing a length would turn one bad frame into a
        // stream of nonsense.
        if total < 8 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("event frame announced {total} bytes"),
            ));
        }
        let rest = total - 8;
        if rest > 0 {
            let more = self.read_exactly_cancellable(rest, running)?;
            frame.extend_from_slice(&more);
        }
        Ok(frame)
    }

    /// Turn one event frame into a row.
    ///
    /// `Ok(None)` means a well-formed frame we have no use for (a door or system
    /// event), so the caller loops.
    fn decode_event(&mut self, frame: &[u8]) -> Result<Option<AttendanceRow>> {
        if frame.len() < 16 {
            return Ok(None);
        }
        let code = u16::from_le_bytes([frame[8], frame[9]]);
        let session = u16::from_le_bytes([frame[12], frame[13]]);
        let reply = u16::from_le_bytes([frame[14], frame[15]]);
        self.reply = reply;
        if code != CMD_REG_EVENT {
            return Ok(None);
        }
        // Every CMD_REG_EVENT frame carries the same attendance layout, whatever
        // the session field holds. Gating on specific session numbers was guesswork
        // taken from a wire capture: real punches arrive with session values that
        // were not on that list and were silently dropped.
        log::debug!("live_capture: REG_EVENT session={session} reply={reply}");
        Ok(parse_event_payload(&frame[16..]))
    }
}

impl Drop for ZkPull {
    /// Release the device's single session slot on EVERY exit path.
    ///
    /// This device accepts one client at a time: while a session is open, no other
    /// client can connect, and the slot only frees on a proper CMD_EXIT or after
    /// the device decides on its own. `pull_attendance` reaches `finish_read`
    /// only when the read succeeds, so before this a mid-read failure dropped the
    /// socket without the exit command, left the device holding an orphaned
    /// session, and every later client - including the operator's own desktop app
    /// - was reset on accept. That is what makes a failure sticky.
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        // Cap the time here: Drop must not block a worker thread for the full
        // session timeout just to say goodbye.
        let _ = self.stream.set_read_timeout(Some(DROP_TIMEOUT));
        let _ = self.stream.set_write_timeout(Some(DROP_TIMEOUT));
        // Best effort only: the socket may already be broken.
        let _ = self.round_trip(CMD_DATA_WRRQ, &[], 0);
        let _ = self.round_trip(CMD_EXIT, &[], 0);
    }
}

// Device time codec: second, minute, hour, day %31, month %12, year t/12+2000.
// Order matters: year-first yields plausible but absurd dates like 0009-09-08.
pub fn decode_device_time(raw: u32) -> (i32, u32, u32, u32, u32, u32) {
    let mut t = raw;
    let second = t % 60;
    t /= 60;
    let minute = t % 60;
    t /= 60;
    let hour = t % 24;
    t /= 24;
    let day = t % 31 + 1;
    t /= 31;
    let month = t % 12 + 1;
    let year = (t / 12) as i32 + 2000;
    (year, month, day, hour, minute, second)
}

/// Decode one real-time event payload into a row.
///
/// A live punch carries less than a table record: no `user_id` string, and the
/// employee is identified by the device-side uid alone. The layout is uid, state,
/// timestamp, punch type, so the field offsets differ from the 40-byte table
/// record and cannot share its decoder.
fn parse_event_payload(payload: &[u8]) -> Option<AttendanceRow> {
    // The wire sends one record per 4/12/32/36/52 byte group, and the record width
    // is what tells you where the timestamp sits. Every width past the 12-byte one
    // carries the user id as a fixed 24-byte null-padded string, not a 4-byte
    // integer, so the same offsets do not apply to all of them.
    let (user_id, status, punch, timehex) = match payload.len() {
        12 => {
            let uid = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
            (uid.to_string(), payload[4], payload[5], &payload[6..12])
        }
        32 => (read_user_id(&payload[..24]), payload[24], payload[25], &payload[26..32]),
        36 => (read_user_id(&payload[..24]), payload[24], payload[25], &payload[26..32]),
        n if n >= 52 => (read_user_id(&payload[..24]), payload[24], payload[25], &payload[26..32]),
        // A partial or unknown width means the stream is mid-record, not that the
        // punch is unusable: skip it and let the next frame resynchronise.
        _ => return None,
    };

    if user_id.is_empty() {
        return None;
    }
    let device_time = pack_device_time(timehex);
    Some(AttendanceRow {
        user_id,
        device_time,
        status,
        punch,
    })
}

/// Read the fixed-width null-padded user id string.
///
/// The device sends the id as a 24-byte field, so the numeric part ends at the
/// first null. Reading it as a 4-byte integer instead turns user "2" into 50,
/// because the ASCII bytes `\x32\x00\x00\x00` read little-endian as 50.
fn read_user_id(raw: &[u8]) -> String {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).trim().to_string()
}

/// Pack the device's 6-byte calendar stamp into the packed integer the store
/// path expects.
///
/// The wire format is six plain binary bytes - year offset from 2000, month, day,
/// hour, minute, second - and `decode_device_time` already knows how to expand
/// that packing. This function only has to invert the device's byte order into it,
/// so it must NOT return an epoch value: doing so made the store path decode the
/// epoch a second time and every punch landed in 2029.
fn pack_device_time(stamp: &[u8]) -> u32 {
    if stamp.len() < 6 {
        return 0;
    }
    let year = stamp[0] as u32;
    let month = stamp[1] as u32;
    let day = stamp[2] as u32;
    let hour = stamp[3] as u32;
    let minute = stamp[4] as u32;
    let second = stamp[5] as u32;

    // Mirrors decode_device_time exactly: day is 1-based and stored as day-1,
    // month likewise, so the two are inverses of each other.
    (((year * 12 + (month - 1)) * 31 + (day - 1)) * 24 + hour) * 3600 + minute * 60 + second
}

pub fn decode_records(body: &[u8], record_len: usize) -> Vec<AttendanceRow> {
    let mut rows = Vec::new();
    // `body` is the record area with the 4-byte size word already stripped by the
    // caller. Stripping it again here shifted every row by four bytes and read
    // the timestamp out of the wrong field, which is why valid records were
    // rejected at insert.
    let mut rest = body;
    while rest.len() >= record_len {
        let rec = &rest[..record_len];
        rest = &rest[record_len..];
        if record_len == RECORD_40 {
            let mut name = [0u8; 24];
            name.copy_from_slice(&rec[2..26]);
            let end = name.iter().position(|&b| b == 0).unwrap_or(24);
            let stamp = u32::from_le_bytes([rec[27], rec[28], rec[29], rec[30]]);
            rows.push(AttendanceRow {
                user_id: String::from_utf8_lossy(&name[..end]).into_owned(),
                device_time: stamp,
                status: rec[26],
                punch: rec[31],
            });
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_device_time_round_trips_through_decode() {
        // 2026-10-11 15:59:48 UTC, the timestamp a real punch carried.
        let stamp = [26u8, 10, 11, 15, 59, 48];
        let packed = pack_device_time(&stamp);
        let (year, month, day, hour, minute, second) = decode_device_time(packed);
        assert_eq!(
            (year, month, day, hour, minute, second),
            (2026, 10, 11, 15, 59, 48)
        );
    }

    #[test]
    fn parse_event_payload_reads_the_36_byte_record() {
        // Captured off device 10.55.54.188 during a real punch: a 36-byte record
        // whose user id is the 24-byte string "2", not a 4-byte integer. Reading
        // those bytes as an integer gives 50, which is how the wrong employee and
        // the year 2029 got in.
        let payload = [
            0x32u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,             // status
            0,             // punch
            26, 10, 11, 22, 8, 0, // 2026-10-11 22:08:00
            0, 0, 0, 0,    // trailing field
        ];
        let row = parse_event_payload(&payload).expect("record should decode");
        assert_eq!(row.user_id, "2", "user id must come from the string field");
        let (year, month, day, hour, minute, second) = decode_device_time(row.device_time);
        assert_eq!(
            (year, month, day, hour, minute, second),
            (2026, 10, 11, 22, 8, 0),
            "the stamp must survive the pack/decode round trip intact"
        );
    }

    #[test]
    fn read_user_id_trims_the_null_padding() {
        let raw = [b'7', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(read_user_id(&raw), "7");
        assert_eq!(read_user_id(b"12"), "12");
        assert_eq!(read_user_id(b""), "");
    }

    #[test]
    fn checksum_matches_wire_capture() {
        // CMD_CONNECT, reply 65534: the live capture on this device showed
        // the checksum word as 17 fc, i.e. 0xFC17 little-endian.
        let mut head = Vec::new();
        head.extend_from_slice(&1000u16.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&65534u16.to_le_bytes());
        assert_eq!(checksum(&head), 0xFC17);
    }

    #[test]
    fn device_time_decodes_sane_dates() {
        let (year, month, day, hour, minute, second) = decode_device_time(0);
        assert_eq!(
            (year, month, day, hour, minute, second),
            (2000, 1, 1, 0, 0, 0)
        );
        // A 2026 timestamp must not decode to a far-past or far-future year.
        let stamp = (((((26u32 * 12 + 9) * 31 + 9) * 24 + 8) * 60 + 30) * 60) + 15;
        let (year, month, day, _, _, _) = decode_device_time(stamp);
        assert_eq!((year, month, day), (2026, 10, 10));
    }

    #[test]
    fn record_40_parses_one_row() {
        let mut rec = vec![0u8; 40];
        rec[0..2].copy_from_slice(&7u16.to_le_bytes());
        rec[2..6].copy_from_slice(b"42\x00\x00");
        rec[26] = 1;
        rec[27..31].copy_from_slice(&12345u32.to_le_bytes());
        rec[31] = 0;
        // The caller strips the 4-byte size word, so the decoder gets records
        // only. This test used to prepend a header and rely on the decoder
        // stripping it a second time, which hid an off-by-four in production.
        let rows = decode_records(&rec, RECORD_40);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].user_id, "42");
        assert_eq!(rows[0].device_time, 12345);
    }

    #[test]
    fn decoder_does_not_strip_a_second_header() {
        // Two records, no header. A decoder that skipped four more bytes would
        // see only half a record and return nothing.
        let mut body = vec![0u8; 40];
        body[2..6].copy_from_slice(b"42\x00\x00");
        let second = body.clone();
        body.extend_from_slice(&second);

        let rows = decode_records(&body, RECORD_40);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].user_id, "42");
    }

    // The session slot is released by Drop on every path, but Drop must never block
    // for the full read timeout just to say goodbye. This guards that cap, which
    // is the one property a unit test can check without the device.
    #[test]
    fn drop_timeout_is_bounded() {
        assert!(DROP_TIMEOUT >= Duration::from_secs(1));
        assert!(DROP_TIMEOUT < Duration::from_secs(180));
    }
}
