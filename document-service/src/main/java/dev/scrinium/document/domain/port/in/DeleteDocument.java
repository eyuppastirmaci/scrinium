package dev.scrinium.document.domain.port.in;

import java.util.UUID;

public interface DeleteDocument {
    void delete(UUID documentId);
}
