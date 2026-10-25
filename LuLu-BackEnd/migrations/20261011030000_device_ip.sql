-- Per-device LAN address, needed by the pull reader.
--
-- Nothing stored it before: the ADMS push path makes the device dial the
-- server, so its own IP never had to be known. Pulling attendance the other
-- way (server dials the device over TCP 4370) requires it, and so does a
-- remote restart, which is the only way out of the stuck-session state the
-- device falls into after a dropped bulk read.
--
-- Sized from the widest address any deployment needs rather than the widest
-- one seen so far: the longest legal IPv4 literal is 15 characters
-- ("255.255.255.255"), and an IPv6 literal with the usual compressed form
-- is far longer, so the column is wide enough for both.
ALTER TABLE devices ADD COLUMN IF NOT EXISTS device_ip VARCHAR(45);

-- The device this column was seeded from, so the restart button is usable
-- immediately instead of after four manual edits.
UPDATE devices
   SET device_ip = '10.55.54.188'
 WHERE device_ip IS NULL
   AND device_sn = 'ADGE233560';