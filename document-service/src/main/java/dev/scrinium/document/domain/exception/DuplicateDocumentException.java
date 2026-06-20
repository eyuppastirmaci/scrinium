package dev.scrinium.document.domain.exception;

import java.util.UUID;

public class DuplicateDocumentException extends DomainException {

    private final UUID existingDocumentId;

    public DuplicateDocumentException(UUID existingDocumentId) {
        super("A document with the same content already exists: " + existingDocumentId);
        this.existingDocumentId = existingDocumentId;
    }

    public UUID existingDocumentId() {
        return existingDocumentId;
    }
}
