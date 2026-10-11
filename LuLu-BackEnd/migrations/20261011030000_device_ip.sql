ALTER TABLE devices ADD COLUMN IF NOT EXISTS device_ip VARCHAR(45);

UPDATE devices
   SET device_ip = '10.55.54.188'
 WHERE device_ip IS NULL
   AND device_sn = 'ADGE233560';