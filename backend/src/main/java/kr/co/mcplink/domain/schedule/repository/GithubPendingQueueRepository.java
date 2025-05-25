package kr.co.mcplink.domain.schedule.repository;

import kr.co.mcplink.domain.schedule.entity.GithubPendingQueue;
import org.springframework.data.mongodb.repository.MongoRepository;
import org.springframework.data.mongodb.repository.Query;
import org.springframework.data.mongodb.repository.Update;
import org.springframework.stereotype.Repository;

import java.util.List;

@Repository
public interface GithubPendingQueueRepository extends MongoRepository<GithubPendingQueue, String> {

    boolean existsByName(String name);

    long countByProcessedFalse();

    List<GithubPendingQueue> findTop10ByProcessedFalseOrderBySeqAsc();

    @Query("{ '_id': ?0 }")
    @Update("{ '$set': { 'processed': ?1, 'updated_at': new Date() } }")
    long updateProcessedById(String _id, boolean processed);
}