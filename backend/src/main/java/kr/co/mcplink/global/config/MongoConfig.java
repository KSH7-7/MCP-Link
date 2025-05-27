package kr.co.mcplink.global.config;

import com.mongodb.ConnectionString;
import com.mongodb.MongoClientSettings;
import com.mongodb.ServerApi;
import com.mongodb.ServerApiVersion;
import com.mongodb.client.MongoClient;
import com.mongodb.client.MongoClients;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Profile;
import org.springframework.core.convert.converter.Converter;
import org.springframework.data.convert.Jsr310Converters;
import org.springframework.data.mongodb.MongoDatabaseFactory;
import org.springframework.data.mongodb.MongoTransactionManager;
import org.springframework.data.mongodb.config.EnableMongoAuditing;
import org.springframework.data.mongodb.core.MongoTemplate;
import org.springframework.data.mongodb.core.SimpleMongoClientDatabaseFactory;
import org.springframework.data.mongodb.core.convert.DefaultDbRefResolver;
import org.springframework.data.mongodb.core.convert.DefaultMongoTypeMapper;
import org.springframework.data.mongodb.core.convert.MappingMongoConverter;
import org.springframework.data.mongodb.core.convert.MongoCustomConversions;
import org.springframework.data.mongodb.core.mapping.MongoMappingContext;
import org.springframework.data.mongodb.repository.config.EnableMongoRepositories;

import java.util.ArrayList;
import java.util.Collection;

@Configuration
@Profile("!test") // 테스트 할 땐 비활성화
@EnableMongoAuditing
@EnableMongoRepositories(
        basePackages = {
                "kr.co.mcplink.domain.mcpserver.repository",
                "kr.co.mcplink.domain.schedule.repository"
        },
        mongoTemplateRef = "mongoTemplate"
)
public class MongoConfig {

    @Value("${mongodb.atlas.uri}")
    private String mongoUri;

    @Value("${mongodb.atlas.database}")
    private String database;

    @Bean
    public MongoClient mongoClient() {
        MongoClientSettings settings = MongoClientSettings.builder()
                .applyConnectionString(new ConnectionString(mongoUri))
                .serverApi(ServerApi.builder().version(ServerApiVersion.V1).build())
                .build();
        return MongoClients.create(settings);
    }

    @Bean
    public MongoDatabaseFactory mongoDatabaseFactory(MongoClient mongoClient) {
        return new SimpleMongoClientDatabaseFactory(mongoClient, database);
    }

    @Bean
    public MongoTemplate mongoTemplate(MongoDatabaseFactory factory, MongoMappingContext context) {
        MappingMongoConverter converter = new MappingMongoConverter(new DefaultDbRefResolver(factory), context);
        converter.setTypeMapper(new DefaultMongoTypeMapper(null));

        Collection<Converter<?, ?>> jsr310Converters = Jsr310Converters.getConvertersToRegister();
        converter.setCustomConversions(new MongoCustomConversions(new ArrayList<>(jsr310Converters)));
        converter.afterPropertiesSet();

        return new MongoTemplate(factory, converter);
    }

    @Bean
    public MongoTransactionManager mongoTransactionManager(MongoDatabaseFactory factory) {
        return new MongoTransactionManager(factory);
    }
}