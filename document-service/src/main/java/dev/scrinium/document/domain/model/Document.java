package dev.scrinium.document.domain.model;

import dev.scrinium.document.domain.exception.InvalidDocumentException;

import java.time.OffsetDateTime;
import java.util.Objects;
import java.util.UUID;

public record Document(
        UUID id,
        String fileName,
        DocumentStatus status,
        OffsetDateTime createdAt,
        OffsetDateTime updatedAt
) {
    public Document {
        Objects.requireNonNull(id, "id");
        Objects.requireNonNull(status, "status");
        Objects.requireNonNull(createdAt, "createdAt");
        Objects.requireNonNull(updatedAt, "updatedAt");

        if (fileName == null || fileName.isBlank()) {
            throw new InvalidDocumentException("fileName must not be blank");
        }
    }

    public static Document pending(UUID id, String fileName, OffsetDateTime now) {
        return new Document(id, fileName, DocumentStatus.PENDING, now, now);
    }
}