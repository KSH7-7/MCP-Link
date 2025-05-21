document.addEventListener('DOMContentLoaded', async () => {
    let isLoggedIn = false;
    if (typeof window.ensureLoginStateKnown === 'function') {
        try {
            isLoggedIn = await window.ensureLoginStateKnown(); 
        } catch (error) {
            console.error("ensureLoginStateKnown 실행 중 오류 (create-post.js):", error);
            alert(translate('errorLoginStatusCheck') || '로그인 상태 확인 중 오류가 발생했습니다. 메인 페이지로 이동합니다.');
            window.location.href = 'index.html';
            return;
        }
    } else {
        console.error("window.ensureLoginStateKnown 함수를 찾을 수 없습니다 (create-post.js).");
        alert(translate('unableToVerifyLoginStatus') || '로그인 상태를 확인할 수 없습니다. 메인 페이지로 이동합니다.');
        window.location.href = 'index.html';
        return;
    }

    if (!isLoggedIn) { 
        alert(translate('loginRequiredRedirect') || '로그인이 필요한 페이지입니다. 메인 페이지로 이동합니다.');
        window.location.href = 'index.html';
        return; 
    }

    const createPostForm = document.getElementById('create-post-form');
    const titleInput = document.getElementById('post-title-input');
    const contentInput = document.getElementById('post-content-input');
    const submitButton = createPostForm.querySelector('button[type="submit"]');

    if (createPostForm) {
        createPostForm.addEventListener('submit', async (event) => {
            event.preventDefault(); // 폼 기본 제출 동작 방지

            const title = titleInput.value.trim();
            const content = contentInput.value.trim();

            if (!title || !content) {
                alert(translate('fillAllFields') || '제목과 내용을 모두 입력해주세요.'); // 간단한 유효성 검사
                return;
            }

            // 제출 버튼 비활성화 및 로딩 표시
            submitButton.disabled = true;
            submitButton.textContent = translate('submittingPost') || '등록 중...';

            try {
                const response = await fetch(`${API_BASE_URL}/v1/posts`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    credentials: 'include',
                    body: JSON.stringify({ 
                        title: title,
                        content: content 
                    })
                });

                if (!response.ok) {
                    const errorData = await response.json().catch(() => null);
                    const serverErrorMessage = errorData?.message || `HTTP error! status: ${response.status}`;
                    throw new Error(serverErrorMessage);
                }

                alert(translate('postCreatedSuccessfully') || '게시글이 성공적으로 등록되었습니다.');
                window.location.href = 'board.html'; // 게시판 목록으로 리디렉션

            } catch (error) {
                console.error('Error creating post:', error);
                const baseMessage = translate('postCreationFailed') || '게시글 등록에 실패했습니다: ';
                alert(baseMessage + error.message);

                // 제출 버튼 다시 활성화
                submitButton.disabled = false;
                submitButton.textContent = translate('createPostFormSubmitButton') || '등록하기';
            }
        });
    }
    // 페이지 로드 시 meta description 번역 및 설정
    const metaDescription = document.querySelector('meta[name="description"]');
    if (metaDescription) {
        metaDescription.setAttribute('content', translate('createPostMetaDescription') || 'mcplink 게시판에 새 글을 작성합니다.');
    }
}); 