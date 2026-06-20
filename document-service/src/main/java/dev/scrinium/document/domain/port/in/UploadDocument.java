package dev.scrinium.document.domain.port.in;

import dev.scrinium.document.domain.model.Document;

import java.io.InputStream;

public interface UploadDocument {
    Document upload(Command command);

    record Command(
            String fileName,
            String contentType,
            long sizeBytes,
            InputStream content
    ) {}
}
