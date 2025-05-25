package kr.co.mcplink.domain.post.entity;

import jakarta.persistence.*;
import kr.co.mcplink.domain.comment.entity.Comment;
import kr.co.mcplink.domain.user.entity.User;
import kr.co.mcplink.global.common.BaseTimeEntity;
import lombok.AccessLevel;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

import java.util.ArrayList;
import java.util.List;

@Entity
@Getter
@NoArgsConstructor(access = AccessLevel.PROTECTED)
@Table(name = "posts")
public class Post extends BaseTimeEntity {

	@Id
	@GeneratedValue(strategy = GenerationType.IDENTITY)
	@Column(name = "post_id")
	private Long id;

	@Column(nullable = false, length = 100)
	private String title;

	@Lob
	@Column(nullable = false, columnDefinition = "TEXT")
	private String content;

	@ManyToOne(fetch = FetchType.LAZY)
	@JoinColumn(name = "user_id")
	private User user;

	@Column(nullable = false, columnDefinition = "integer default 0")
	private Integer likeCount = 0;

	@Column(nullable = false, columnDefinition = "integer default 0")
	private Integer viewCount = 0;

	@OneToMany(mappedBy = "post", cascade = CascadeType.ALL, orphanRemoval = true, fetch = FetchType.LAZY)
	private List<Comment> comments = new ArrayList<>();

	@Builder
	private Post(String title, String content, User user) {
		this.title = title;
		this.content = content;
		this.user = user;
		this.likeCount = 0;
	}

	public static Post createPost(String title, String content, User user) {
		return Post.builder()
			.title(title)
			.content(content)
			.user(user)
			.build();
	}

	public void updatePost(String title, String content) {
		this.title = title == null ? this.title : title;
		this.content = content == null ? this.content : content;
	}

	public void incrementLikeCount() {
		this.likeCount++;
	}

	public void decrementLikeCount() {
		if (this.likeCount > 0) {
			this.likeCount--;
		}
	}

	public void incrementViewCount() {
		this.viewCount++;
	}
}