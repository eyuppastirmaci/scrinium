package dev.scrinium.document.adapter.in.sse;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.data.redis.connection.Message;
import org.springframework.data.redis.connection.MessageListener;
import org.springframework.data.redis.listener.ChannelTopic;
import org.springframework.data.redis.listener.RedisMessageListenerContainer;
import org.springframework.stereotype.Service;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;

@Service
public class ProcessingProgressService implements MessageListener {

    private static final Logger log = LoggerFactory.getLogger(ProcessingProgressService.class);

    private final List<SseEmitter> emitters = new CopyOnWriteArrayList<>();

    public ProcessingProgressService(RedisMessageListenerContainer listenerContainer) {
        listenerContainer.addMessageListener(this, new ChannelTopic("doc:progress"));
        log.info("Subscribed to Redis channel 'doc:progress' for SSE broadcasting");
    }

    public SseEmitter subscribe() {
        SseEmitter emitter = new SseEmitter(0L);
        emitters.add(emitter);

        emitter.onCompletion(() -> emitters.remove(emitter));
        emitter.onTimeout(() -> emitters.remove(emitter));
        emitter.onError(e -> emitters.remove(emitter));

        log.debug("SSE client connected, total: {}", emitters.size());
        return emitter;
    }

    @Override
    public void onMessage(Message message, byte[] pattern) {
        String json = new String(message.getBody(), StandardCharsets.UTF_8);
        log.debug("Redis progress event: {}", json);

        for (SseEmitter emitter : emitters) {
            try {
                emitter.send(SseEmitter.event()
                        .name("progress")
                        .data(json));
            } catch (IOException e) {
                emitters.remove(emitter);
            }
        }
    }
}
