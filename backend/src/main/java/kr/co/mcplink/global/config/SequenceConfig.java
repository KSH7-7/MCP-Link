package kr.co.mcplink.global.config;

import kr.co.mcplink.domain.mcpserver.service.storage.SequenceGeneratorService;
import kr.co.mcplink.global.annotation.AutoSequence;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.event.EventListener;
import org.springframework.data.mongodb.core.mapping.event.BeforeConvertEvent;
import org.springframework.stereotype.Component;

import java.lang.reflect.Field;

@Component
public class SequenceConfig {
    @Autowired
    private SequenceGeneratorService sequenceGeneratorService;

    @EventListener
    public void onBeforeConvert(BeforeConvertEvent<Object> event) {
        Object source = event.getSource();
        Class<?> clazz = source.getClass();
        if (clazz.isAnnotationPresent(AutoSequence.class)) {
            AutoSequence ann = clazz.getAnnotation(AutoSequence.class);
            String collectionName = ann.collection();
            try {
                Field seqField = clazz.getDeclaredField("seq");
                seqField.setAccessible(true);
                Object current = seqField.get(source);
                if (current == null || ((Long) current) <= 0) {
                    long next = sequenceGeneratorService.generateSequence(collectionName);
                    seqField.set(source, next);
                }
            } catch (NoSuchFieldException | IllegalAccessException e) {
                throw new RuntimeException("Failed to set seq for " + clazz.getSimpleName(), e);
            }
        }
    }
}