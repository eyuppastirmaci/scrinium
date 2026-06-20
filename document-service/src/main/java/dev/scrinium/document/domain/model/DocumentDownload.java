package dev.scrinium.document.domain.model;

import java.io.InputStream;

public record DocumentDownload(
        String fileName,
        String contentType,
        long sizeBytes,
        InputStream content
) {}
