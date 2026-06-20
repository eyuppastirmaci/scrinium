package dev.scrinium.document.domain.port.out;

import dev.scrinium.document.domain.model.DocumentFile;
import dev.scrinium.document.domain.model.StoredDocument;

public interface DocumentStorage {
    StoredDocument store(DocumentFile file);
}
