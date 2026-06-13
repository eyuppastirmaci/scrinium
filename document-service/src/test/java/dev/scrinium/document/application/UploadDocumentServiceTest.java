package dev.scrinium.document.application;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.model.DocumentStatus;
import dev.scrinium.document.domain.port.in.UploadDocument;
import dev.scrinium.document.domain.port.out.DocumentEventPublisher;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class UploadDocumentServiceTest {

    @Mock
    DocumentRepository repository;

    @Mock
    DocumentEventPublisher eventPublisher;

    @InjectMocks
    UploadDocumentService service;

    @Test
    void upload_createsPendingDocumentWithGivenFileName() {
        Document result = service.upload(new UploadDocument.Command("invoice.pdf"));

        assertThat(result.fileName()).isEqualTo("invoice.pdf");
        assertThat(result.status()).isEqualTo(DocumentStatus.PENDING);
        assertThat(result.id()).isNotNull();
    }

    @Test
    void upload_persistsTheSameDocumentItPublishes() {
        service.upload(new UploadDocument.Command("invoice.pdf"));

        // Capture what each port received and assert they are the same aggregate.
        ArgumentCaptor<Document> saved = ArgumentCaptor.forClass(Document.class);
        ArgumentCaptor<Document> published = ArgumentCaptor.forClass(Document.class);
        verify(repository).save(saved.capture());
        verify(eventPublisher).documentUploaded(published.capture());

        assertThat(saved.getValue().id()).isEqualTo(published.getValue().id());
    }

    @Test
    void upload_savesBeforePublishing() {
        service.upload(new UploadDocument.Command("invoice.pdf"));

        // Order matters: the row must exist before we announce it.
        var ordered = inOrder(repository, eventPublisher);
        ordered.verify(repository).save(org.mockito.ArgumentMatchers.any());
        ordered.verify(eventPublisher).documentUploaded(org.mockito.ArgumentMatchers.any());
    }
}