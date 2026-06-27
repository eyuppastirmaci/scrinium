package dev.scrinium.document.domain.port.in;

import java.util.UUID;

public interface RetryProcessing {
    void retry(UUID documentId);
}
