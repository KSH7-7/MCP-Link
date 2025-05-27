package kr.co.mcplink.domain.user.entity;

import jakarta.persistence.*;
import kr.co.mcplink.global.common.BaseTimeEntity;
import kr.co.mcplink.global.common.UserRole;
import lombok.AccessLevel;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

import java.time.LocalDateTime;

@Entity
@Table(name = "users")
@Getter
@NoArgsConstructor(access = AccessLevel.PROTECTED)
public class User extends BaseTimeEntity {

	@Id
	@GeneratedValue(strategy = GenerationType.IDENTITY)
	@Column(name = "user_id")
	private Long id;

	@Column(name = "name")
	private String name;

	@Column(name = "email", nullable = false)
	private String email;

	@Column(name = "ssafy_user_id")
	private String ssafyUserId;

	@Column(name = "nickname")
	private String nickname;

	@Column(name = "role")
	@Enumerated(EnumType.STRING)
	private UserRole role;

	@Column(name = "last_login_at")
	private LocalDateTime lastLoginAt;

	@Builder
	private User(String name, String email, String nickname, String ssafyUserId, UserRole role) {
		this.name = name;
		this.email = email;
		this.nickname = nickname;
		this.ssafyUserId = ssafyUserId;
		this.role = role;
	}

	public void updateSsafyUser(String name, String ssafyUserId) {
		this.name = name;
		this.ssafyUserId = ssafyUserId;
	}

	public void updateLastLoginAt() {
		this.lastLoginAt = LocalDateTime.now();
	}

	public static User createUser(String name, String nickname,String email) {
		return User.builder()
			.name(name)
			.email(email)
			.nickname(nickname == null ? name : nickname)
			.role(UserRole.USER)
			.build();
	}

	public static User createSsafyUser(String name, String email, String ssafyUserId) {
		return User.builder()
			.name(name)
			.email(email)
			.nickname(name)
			.ssafyUserId(ssafyUserId)
			.role(UserRole.USER)
			.build();
	}

}
