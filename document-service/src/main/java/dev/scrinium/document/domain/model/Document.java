package dev.scrinium.document.domain.model;

import dev.scrinium.document.domain.exception.InvalidDocumentException;

import java.time.OffsetDateTime;
import java.util.Objects;
import java.util.UUID;

public record Document(
        UUID id,
        String fileName,
        String contentType,
        long sizeBytes,
        String storageObjectKey,
        String sha256,
        DocumentStatus status,
        String failureReason,
        OffsetDateTime createdAt,
        OffsetDateTime updatedAt
) {
    public Document {
        Objects.requireNonNull(id, "id");
        Objects.requireNonNull(contentType, "contentType");
        Objects.requireNonNull(storageObjectKey, "storageObjectKey");
        Objects.requireNonNull(status, "status");
        Objects.requireNonNull(createdAt, "createdAt");
        Objects.requireNonNull(updatedAt, "updatedAt");

        if (fileName == null || fileName.isBlank()) {
            throw new InvalidDocumentException("fileName must not be blank");
        }
        if (contentType.isBlank()) {
            throw new InvalidDocumentException("contentType must not be blank");
        }
        if (sizeBytes <= 0) {
            throw new InvalidDocumentException("sizeBytes must be positive");
        }
        if (storageObjectKey.isBlank()) {
            throw new InvalidDocumentException("storageObjectKey must not be blank");
        }
        if (sha256 != null && sha256.isBlank()) {
            throw new InvalidDocumentException("sha256 must not be blank when present");
        }
        if (failureReason != null && failureReason.isBlank()) {
            throw new InvalidDocumentException("failureReason must not be blank when present");
        }
    }

    public static Document pending(
            UUID id,
            String fileName,
            String contentType,
            long sizeBytes,
            String storageObjectKey,
            String sha256,
            OffsetDateTime now
    ) {
        return new Document(
                id,
                fileName,
                contentType,
                sizeBytes,
                storageObjectKey,
                sha256,
                DocumentStatus.PENDING,
                null,
                now,
                now
        );
    }
}
