CREATE TABLE extracted_pages (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES processing_jobs(document_id),
    page_number INTEGER NOT NULL,
    extracted_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT extracted_pages_page_number_positive
        CHECK (page_number > 0),
    CONSTRAINT extracted_pages_unique_page
        UNIQUE (document_id, page_number)
);

CREATE INDEX extracted_pages_document_id_idx ON extracted_pages (document_id);
