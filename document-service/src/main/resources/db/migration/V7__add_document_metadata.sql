CREATE TABLE document_metadata (
    document_id UUID PRIMARY KEY REFERENCES documents(id) ON DELETE CASCADE,
    page_count INTEGER,
    pdf_created_at TIMESTAMPTZ,
    pdf_modified_at TIMESTAMPTZ,
    pdf_author TEXT,
    pdf_title TEXT,
    image_captured_at TIMESTAMPTZ,
    image_device TEXT,
    image_gps_present BOOLEAN NOT NULL DEFAULT false,
    image_gps_redacted BOOLEAN NOT NULL DEFAULT false,
    detected_language TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
