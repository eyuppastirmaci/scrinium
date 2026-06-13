package dev.scrinium.document.domain.port.in;

import dev.scrinium.document.domain.model.Document;

public interface UploadDocument {
    Document upload(Command command);

    record Command(String fileName) {}
}
