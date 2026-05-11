ALTER TABLE plan_reason_cancel
ADD COLUMN check_reason_id_old smallint[] DEFAULT '{}' NOT NULL;

UPDATE plan_reason_cancel
SET check_reason_id_old = ARRAY[check_reason_id];

ALTER TABLE plan_reason_cancel
DROP COLUMN check_reason_id;

ALTER TABLE plan_reason_cancel  
RENAME COLUMN check_reason_id_old TO check_reason_id;