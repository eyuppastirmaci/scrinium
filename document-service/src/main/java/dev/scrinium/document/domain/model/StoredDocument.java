package dev.scrinium.document.domain.model;

import dev.scrinium.document.domain.exception.InvalidDocumentException;

import java.util.Objects;

public record StoredDocument(
        String objectKey,
        String contentType,
        long sizeBytes,
        String sha256
) {
    public StoredDocument {
        Objects.requireNonNull(objectKey, "objectKey");
        Objects.requireNonNull(contentType, "contentType");
        Objects.requireNonNull(sha256, "sha256");

        if (objectKey.isBlank()) {
            throw new InvalidDocumentException("objectKey must not be blank");
        }
        if (contentType.isBlank()) {
            throw new InvalidDocumentException("contentType must not be blank");
        }
        if (sizeBytes <= 0) {
            throw new InvalidDocumentException("sizeBytes must be positive");
        }
        if (sha256.isBlank()) {
            throw new InvalidDocumentException("sha256 must not be blank");
        }
    }
}
