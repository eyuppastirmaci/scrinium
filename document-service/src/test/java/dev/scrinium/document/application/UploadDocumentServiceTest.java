package dev.scrinium.document.application;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.model.DocumentFile;
import dev.scrinium.document.domain.model.DocumentStatus;
import dev.scrinium.document.domain.model.StoredDocument;
import dev.scrinium.document.domain.port.in.UploadDocument;
import dev.scrinium.document.domain.port.out.DocumentEventPublisher;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import dev.scrinium.document.domain.port.out.DocumentStorage;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.ByteArrayInputStream;
import java.nio.charset.StandardCharsets;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class UploadDocumentServiceTest {

    @Mock
    DocumentRepository repository;

    @Mock
    DocumentStorage storage;

    @Mock
    DocumentEventPublisher eventPublisher;

    @InjectMocks
    UploadDocumentService service;

    @Test
    void upload_createsPendingDocumentFromStoredFileMetadata() {
        when(storage.store(org.mockito.ArgumentMatchers.any()))
                .thenReturn(storedDocument());

        Document result = service.upload(command());

        assertThat(result.fileName()).isEqualTo("invoice.pdf");
        assertThat(result.contentType()).isEqualTo("application/pdf");
        assertThat(result.sizeBytes()).isEqualTo(1_024);
        assertThat(result.storageObjectKey()).isEqualTo("documents/generated/invoice.pdf");
        assertThat(result.sha256()).isEqualTo("abc123");
        assertThat(result.status()).isEqualTo(DocumentStatus.PENDING);
        assertThat(result.id()).isNotNull();
    }

    @Test
    void upload_storesFileBeforePersistingAndPublishingDocument() {
        when(storage.store(org.mockito.ArgumentMatchers.any()))
                .thenReturn(storedDocument());

        service.upload(command());

        ArgumentCaptor<DocumentFile> stored = ArgumentCaptor.forClass(DocumentFile.class);
        var ordered = inOrder(storage, repository, eventPublisher);
        ordered.verify(storage).store(stored.capture());
        ordered.verify(repository).save(org.mockito.ArgumentMatchers.any());
        ordered.verify(eventPublisher).documentUploaded(org.mockito.ArgumentMatchers.any());

        assertThat(stored.getValue().documentId()).isNotNull();
        assertThat(stored.getValue().fileName()).isEqualTo("invoice.pdf");
        assertThat(stored.getValue().contentType()).isEqualTo("application/pdf");
        assertThat(stored.getValue().sizeBytes()).isEqualTo(1_024);
        assertThat(stored.getValue().content()).isNotNull();
    }

    @Test
    void upload_persistsTheSameDocumentItPublishes() {
        when(storage.store(org.mockito.ArgumentMatchers.any()))
                .thenReturn(storedDocument());

        service.upload(command());

        ArgumentCaptor<Document> saved = ArgumentCaptor.forClass(Document.class);
        ArgumentCaptor<Document> published = ArgumentCaptor.forClass(Document.class);
        verify(repository).save(saved.capture());
        verify(eventPublisher).documentUploaded(published.capture());

        assertThat(saved.getValue().id()).isEqualTo(published.getValue().id());
    }

    private UploadDocument.Command command() {
        return new UploadDocument.Command(
                "invoice.pdf",
                "application/pdf",
                1_024,
                new ByteArrayInputStream("pdf bytes".getBytes(StandardCharsets.UTF_8))
        );
    }

    private StoredDocument storedDocument() {
        return new StoredDocument("documents/generated/invoice.pdf", "application/pdf", 1_024, "abc123");
    }
}
