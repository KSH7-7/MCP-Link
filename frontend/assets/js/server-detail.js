document.addEventListener('DOMContentLoaded', () => {
    const serverNameTitle = document.getElementById('server-name-title');
    const serverDetailContent = document.getElementById('server-detail-content');
    const loadingMessage = document.getElementById('loading-message');
    const errorMessage = document.getElementById('error-message');

    // DOM Element-ID mapping
    const elements = {
        url: document.getElementById('server-url'),
        stars: document.getElementById('server-stars'),
        views: document.getElementById('server-views'),
        official: document.getElementById('server-official'),
        scanned: document.getElementById('server-scanned'),
        securityRank: document.getElementById('server-security-rank'),
        type: document.getElementById('server-type'),
        description: document.getElementById('server-description'),
        command: document.getElementById('server-command'),
        args: document.getElementById('server-args'),
        env: document.getElementById('server-env'),
    };

    // Helper function to format large numbers (k/M)
    function formatNumber(num) {
        if (num === undefined || num === null) return translate('notAvailable') || 'N/A';
        if (num >= 1000000) {
            return (num / 1000000).toFixed(1).replace(/\.0$/, '') + 'M';
        }
        if (num >= 1000) {
            return (num / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
        }
        return num.toString();
    }

    // Helper function to display boolean status as icons
    function formatBooleanIcon(value, trueTitle = 'Yes', falseTitle = 'No') {
        if (value === true) {
            return `<i class="bi bi-check-lg text-success" title="${trueTitle}"></i>`;
        }
        return `<i class="bi bi-x-lg text-danger" title="${falseTitle}"></i>`;
    }

    async function fetchServerDetails() {
        const params = new URLSearchParams(window.location.search);
        const serverId = params.get('id');

        if (!serverId) {
            serverNameTitle.textContent = translate('invalidServerIdTitle') || '잘못된 서버 ID';
            loadingMessage.style.display = 'none';
            errorMessage.style.display = 'block';
            errorMessage.firstElementChild.textContent = translate('noServerIdProvided') || '서버 ID가 제공되지 않았습니다.';
            return;
        }

        try {
            const response = await fetch(`${API_BASE_URL}/v3/mcp/servers/${serverId}`);
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const result = await response.json();
            const serverData = result.data?.mcpServer;

            if (!serverData) {
                throw new Error('Server data not found in API response');
            }

            // Populate server name in title and on page
            const serverDisplayName = serverData.mcpServer?.name || 'Unknown Server';
            serverNameTitle.textContent = serverDisplayName;
            document.title = `${serverDisplayName} - ${translate('serverDetailPageTitle') || '서버 상세 정보 - mcplink'}`;
            
            // Populate basic info
            elements.url.href = serverData.url || '#';
            elements.url.textContent = serverData.url ? serverData.url.replace(/^https?:\/\/(www\.)?/, '') : translate('notAvailable') || 'N/A';
            elements.stars.textContent = formatNumber(serverData.stars);
            elements.views.textContent = formatNumber(serverData.views);
            elements.official.innerHTML = formatBooleanIcon(serverData.official, translate('officialServer') || '공식', translate('unofficialServer') || '비공식');
            // HTML에는 "검사 완료"로 되어있으나 API의 scanned는 검사 여부를 뜻하므로, 툴크을 적절히 수정
            elements.scanned.innerHTML = formatBooleanIcon(serverData.scanned, translate('scannedPass') || '검사 통과', translate('scannedFailOrNotYet') || '검사 미통과/미실시');
            elements.securityRank.textContent = serverData.securityRank || translate('notAvailable') || 'N/A';
            elements.type.textContent = serverData.type || translate('notAvailable') || 'N/A';
            
            // Populate description
            elements.description.textContent = serverData.mcpServer?.description || translate('notAvailable') || 'N/A';

            // Populate execution info
            elements.command.textContent = serverData.mcpServer?.command || translate('notAvailable') || 'N/A';

            elements.args.innerHTML = ''; // Clear default
            if (serverData.mcpServer?.args && serverData.mcpServer.args.length > 0) {
                serverData.mcpServer.args.forEach(arg => {
                    const li = document.createElement('li');
                    li.textContent = arg;
                    elements.args.appendChild(li);
                });
            } else {
                const li = document.createElement('li');
                li.textContent = translate('notAvailable') || 'N/A';
                elements.args.appendChild(li);
            }

            elements.env.innerHTML = ''; // Clear default
            if (serverData.mcpServer?.env && Object.keys(serverData.mcpServer.env).length > 0) {
                for (const key in serverData.mcpServer.env) {
                    const li = document.createElement('li');
                    li.innerHTML = `<strong>${key}:</strong> ${serverData.mcpServer.env[key]}`;
                    elements.env.appendChild(li);
                }
            } else {
                const li = document.createElement('li');
                li.textContent = translate('notAvailable') || 'N/A';
                elements.env.appendChild(li);
            }

            serverDetailContent.style.display = 'block';
            loadingMessage.style.display = 'none';

        } catch (error) {
            console.error('Error fetching server details:', error);
            serverNameTitle.textContent = translate('errorPageTitle') || '오류';
            loadingMessage.style.display = 'none';
            errorMessage.style.display = 'block';
        }
    }

    // Ensure translate function is available or provide a fallback
    // This assumes script.js (with translate function) is loaded before this script.
    // If not, a more robust check or a local fallback for translate is needed.
    if (typeof translate !== 'function') {
        console.warn('translate function not found. Using fallback text.');
        window.translate = (key) => key; // Simple fallback
    }

    fetchServerDetails();
}); 