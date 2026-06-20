package dev.scrinium.document.domain.model;

import dev.scrinium.document.domain.exception.InvalidDocumentException;

import java.io.InputStream;
import java.util.Objects;
import java.util.UUID;

public record DocumentFile(
        UUID documentId,
        String fileName,
        String contentType,
        long sizeBytes,
        InputStream content
) {
    public DocumentFile {
        Objects.requireNonNull(documentId, "documentId");
        Objects.requireNonNull(contentType, "contentType");
        Objects.requireNonNull(content, "content");

        if (fileName == null || fileName.isBlank()) {
            throw new InvalidDocumentException("fileName must not be blank");
        }
        if (contentType.isBlank()) {
            throw new InvalidDocumentException("contentType must not be blank");
        }
        if (sizeBytes <= 0) {
            throw new InvalidDocumentException("sizeBytes must be positive");
        }
    }
}
