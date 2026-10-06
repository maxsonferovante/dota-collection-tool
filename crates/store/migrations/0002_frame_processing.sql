ALTER TABLE frames ADD COLUMN payload_kind TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE frames ADD COLUMN normalized_payload TEXT NOT NULL DEFAULT '{}';
