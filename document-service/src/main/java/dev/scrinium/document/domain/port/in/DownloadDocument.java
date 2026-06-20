package dev.scrinium.document.domain.port.in;

import dev.scrinium.document.domain.model.DocumentDownload;

import java.util.UUID;

public interface DownloadDocument {
    DocumentDownload download(UUID documentId);
}
