ALTER TABLE documents ADD COLUMN failure_reason TEXT;

ALTER TABLE documents
    ADD CONSTRAINT documents_failure_reason_not_blank_when_present
        CHECK (failure_reason IS NULL OR btrim(failure_reason) <> '');
