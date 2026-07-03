const { invoke } = window.__TAURI__?.core ?? {};

async function tauriInvoke(cmd, args = {}) {
    if (invoke) return invoke(cmd, args);
    console.warn('Not in Tauri, using mock:', cmd, args);
    return mockInvoke(cmd, args);
}

// === Navigation ===
document.querySelectorAll('.nav-item').forEach(item => {
    item.addEventListener('click', () => {
        document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
        document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
        item.classList.add('active');
        const page = document.getElementById('page-' + item.dataset.page);
        if (page) page.classList.add('active');
        if (item.dataset.page === 'dashboard') loadDashboard();
        if (item.dataset.page === 'watchlist') loadWatchlist();
        if (item.dataset.page === 'plays') loadAllPlays();
        if (item.dataset.page === 'favorites') loadFavorites();
        if (item.dataset.page === 'places') loadPlaces();
        if (item.dataset.page === 'recommendations') initRecommendations();
    });
});

// === Dashboard ===
async function loadDashboard() {
    try {
        const stats = await tauriInvoke('get_dashboard_stats');
        const grid = document.getElementById('stats-grid');
        grid.innerHTML = `
            <div class="stat-card"><div class="stat-value">${stats.movie_total_plays}</div><div class="stat-label">Movie Plays</div></div>
            <div class="stat-card"><div class="stat-value">${stats.tv_total_plays}</div><div class="stat-label">TV Episode Plays</div></div>
            <div class="stat-card"><div class="stat-value">${stats.movie_unique_titles}</div><div class="stat-label">Movies Watched</div></div>
            <div class="stat-card"><div class="stat-value">${stats.tv_unique_shows}</div><div class="stat-label">TV Shows</div></div>
            <div class="stat-card"><div class="stat-value">${stats.movie_watchlist_count}</div><div class="stat-label">Movie Watchlist</div></div>
            <div class="stat-card"><div class="stat-value">${stats.tv_watchlist_count}</div><div class="stat-label">TV Watchlist</div></div>
            <div class="stat-card"><div class="stat-value">${formatRuntime(stats.total_runtime_minutes)}</div><div class="stat-label">Total Time</div></div>
            <div class="stat-card"><div class="stat-value">${stats.total_plays}</div><div class="stat-label">Total Plays</div></div>
        `;

        const plays = await tauriInvoke('get_all_plays');
        const recent = document.getElementById('recent-plays');
        recent.innerHTML = plays.slice(0, 12).map(p => createPlayCard(p)).join('');
    } catch (e) {
        console.error('Dashboard error:', e);
    }
}

function formatRuntime(min) {
    if (!min) return '0m';
    const h = Math.floor(min / 60);
    const m = min % 60;
    return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

// === Search ===
document.getElementById('search-btn').addEventListener('click', doSearch);
document.getElementById('search-input').addEventListener('keydown', e => { if (e.key === 'Enter') doSearch(); });

async function doSearch() {
    const query = document.getElementById('search-input').value.trim();
    if (!query) return;
    const resultsDiv = document.getElementById('search-results');
    const detailDiv = document.getElementById('detail-view');
    detailDiv.style.display = 'none';
    resultsDiv.innerHTML = '<div class="loading"><div class="spinner"></div>Searching...</div>';

    try {
        const results = await tauriInvoke('search_titles', { query });
        if (!results || results.length === 0) {
            resultsDiv.innerHTML = '<p style="color:var(--text2);padding:20px;">No results found.</p>';
            return;
        }
        resultsDiv.innerHTML = results.map(r => {
            const title = r.title || r.name || 'Unknown';
            const year = (r.release_date || r.first_air_date || '').substring(0, 4);
            const poster = r.poster_path ? `https://image.tmdb.org/t/p/w500${r.poster_path}` : null;
            const type = r.media_type === 'tv' ? 'TV' : 'Movie';
            const rating = r.vote_average ? `★ ${r.vote_average.toFixed(1)}` : '';
            return `<div class="card" onclick="showDetail(${r.id}, '${r.media_type}')">
                <div class="card-poster">${poster ? `<img src="${poster}" alt="${title}" loading="lazy">` : '🎬'}</div>
                <div class="card-body">
                    <div class="card-title">${escapeHtml(title)}</div>
                    <div class="card-subtitle">${type} ${year ? '• ' + year : ''} ${rating}</div>
                </div>
            </div>`;
        }).join('');
    } catch (e) {
        resultsDiv.innerHTML = `<p style="color:var(--accent);padding:20px;">Error: ${escapeHtml(e)}</p>`;
    }
}

let currentDetail = null;

async function showDetail(tmdbId, mediaType) {
    const detailDiv = document.getElementById('detail-view');
    detailDiv.style.display = 'block';
    detailDiv.innerHTML = '<div class="loading"><div class="spinner"></div>Loading details...</div>';

    try {
        let data;
        if (mediaType === 'tv') {
            data = await tauriInvoke('get_tv_full', { tmdbId });
        } else {
            data = await tauriInvoke('get_movie_full', { tmdbId });
        }
        currentDetail = { tmdbId, mediaType, data };
        renderDetail(data, mediaType);
    } catch (e) {
        detailDiv.innerHTML = `<p style="color:var(--accent);">Error: ${escapeHtml(e)}</p>`;
    }
}

function renderDetail(data, mediaType) {
    const detailDiv = document.getElementById('detail-view');
    const info = mediaType === 'tv' ? data.show : data.movie;
    const title = info.title || info.name || 'Unknown';
    const originalTitle = info.original_title || info.original_name || '';
    const poster = info.poster || info.poster_path;
    const posterUrl = poster ? `https://image.tmdb.org/t/p/w500${poster}` : null;
    const year = (info.release_date || info.first_air_date || '').substring(0, 4);
    const genres = info.genres || '';
    const runtime = info.runtime;
    const runtimeStr = runtime ? `${Math.floor(runtime/60)}h ${runtime%60}m` : '';
    const lang = info.original_language || '';
    const overview = info.overview || 'No overview available.';
    const tagline = info.tagline || '';
    const rating = info.tmdb_average ? `TMDB ${info.tmdb_average.toFixed(1)}/10` : '';
    const inWatchlist = data.in_watchlist;

    // Group credits
    const credits = data.credits || [];
    const directors = credits.filter(c => c.role_type === 'director');
    const producers = credits.filter(c => c.role_type === 'producer');
    const cast = credits.filter(c => c.role_type === 'actor');
    const composers = credits.filter(c => c.role_type === 'music_composer');

    function renderPeople(people) {
        return people.map(p => {
            const profileUrl = p.profile_path ? `https://image.tmdb.org/t/p/w185${p.profile_path}` : null;
            return `<div class="credit-chip" onclick="viewPerson(${p.tmdb_person_id})">
                ${profileUrl ? `<img src="${profileUrl}" alt="${escapeHtml(p.person_name)}">` :
                    `<div class="credit-avatar-fallback">${p.person_name.charAt(0)}</div>`}
                ${escapeHtml(p.person_name)}${p.character_name ? ` (${escapeHtml(p.character_name)})` : ''}
            </div>`;
        }).join('');
    }

    // Plays
    const plays = data.plays || [];
    const playsHtml = plays.map(p => {
        const d = p.watched_at ? new Date(p.watched_at + 'T00:00:00').toLocaleDateString() : 'No date';
        const ep = p.episode_id ? `S${String(p.season_number).padStart(2,'0')}E${String(p.episode_number).padStart(2,'0')}` : '';
        const stars = renderStars(p.user_rating, p.id, p.source_type);
        return `<div class="play-entry" data-play-id="${p.source_play_id}" data-source-type="${p.source_type}">
            <div><div class="play-date">${d}${ep ? ` <span class="play-episode">${ep} • ${escapeHtml(p.episode_name || '')}</span>` : ''}</div>
            ${p.comment ? `<div style="font-size:12px;color:var(--text2)">${escapeHtml(p.comment)}</div>` : ''}</div>
            <div style="display:flex;align-items:center;gap:8px">
                <div class="play-rating">${stars}</div>
                <div class="play-actions">
                    <button onclick="deletePlayEntry(${p.source_play_id}, '${p.source_type}')" title="Delete">🗑</button>
                </div>
            </div>
        </div>`;
    }).join('');

    // Seasons for TV
    let seasonsHtml = '';
    if (mediaType === 'tv' && data.seasons) {
        seasonsHtml = `<div class="detail-actions">
            <select id="season-select">
                <option value="">-- Add Season Plays --</option>
                ${data.seasons.map(s => `<option value="${s.season_number}">${escapeHtml(s.name || 'Season ' + s.season_number)} (${s.watched_episodes}/${s.total_episodes} watched)</option>`).join('')}
            </select>
            <button class="btn small secondary" onclick="addSeasonPlays()">Log Season</button>
        </div>`;
        const ep = data.episode_progress || {};
        seasonsHtml += `<div style="font-size:13px;color:var(--text2);margin-bottom:8px">
            ${ep.watched_episodes}/${ep.total_episodes} episodes • ${ep.completed_seasons}/${ep.total_seasons} seasons completed
        </div>`;
    }

    detailDiv.innerHTML = `
        <div class="detail-header">
            <div class="detail-poster">${posterUrl ? `<img src="${posterUrl}" alt="${escapeHtml(title)}">` : '🎬'}</div>
            <div class="detail-info">
                <div class="detail-title">${escapeHtml(title)}</div>
                ${tagline ? `<div class="detail-tagline">${escapeHtml(tagline)}</div>` : ''}
                <div class="detail-meta">
                    ${year ? `<span>${year}</span>` : ''}
                    ${runtimeStr ? `<span>${runtimeStr}</span>` : ''}
                    ${lang ? `<span>${lang.toUpperCase()}</span>` : ''}
                    ${rating ? `<span>${rating}</span>` : ''}
                    ${genres ? genres.split(',').map(g => `<span>${g.trim()}</span>`).join('') : ''}
                </div>
                <div class="detail-overview">${escapeHtml(overview)}</div>
                <div class="detail-actions">
                    <button class="btn primary small" onclick="showAddPlayDialog()">➕ Add Play</button>
                    <button class="btn secondary small" onclick="toggleWatchlist(${info.tmdb_id}, '${mediaType}')">
                        ${inWatchlist ? '★ Remove from Watchlist' : '☆ Add to Watchlist'}
                    </button>
                </div>
                ${seasonsHtml}
                ${plays.length > 0 ? `<div class="plays-list">${playsHtml}</div>` :
                    '<div style="color:var(--text2);font-size:14px;margin-top:8px">No plays recorded yet.</div>'}
            </div>
        </div>
        <div class="detail-credits">
            ${directors.length > 0 ? `<div class="credit-group"><h4>Directed By</h4><div class="credit-grid">${renderPeople(directors)}</div></div>` : ''}
            ${producers.length > 0 ? `<div class="credit-group"><h4>Produced By</h4><div class="credit-grid">${renderPeople(producers)}</div></div>` : ''}
            ${composers.length > 0 ? `<div class="credit-group"><h4>Music By</h4><div class="credit-grid">${renderPeople(composers)}</div></div>` : ''}
            ${cast.length > 0 ? `<div class="credit-group"><h4>Cast</h4><div class="credit-grid">${renderPeople(cast)}</div></div>` : ''}
        </div>
        <div id="keywords-section" style="margin-top:12px">
            <h4 style="font-size:14px;color:var(--text2);margin-bottom:8px">Keywords</h4>
            <div id="keywords-list" style="display:flex;flex-wrap:wrap;gap:6px">
                <span style="color:var(--text2);font-size:13px">Loading...</span>
            </div>
        </div>
        <div style="margin-top:12px;display:flex;gap:8px">
            ${info.imdb_id ? `<button class="btn small secondary" onclick="window.open('https://www.imdb.com/title/${info.imdb_id}/','_blank')">IMDb</button>` : ''}
            <button class="btn small secondary" onclick="window.open('https://www.themoviedb.org/${mediaType === 'tv' ? 'tv' : 'movie'}/${info.tmdb_id}','_blank')">TMDB</button>
        </div>`;

    // Load keywords asynchronously
    loadKeywords(info.tmdb_id, mediaType);
}

async function loadKeywords(tmdbId, mediaType) {
    try {
        const keywords = mediaType === 'tv'
            ? await tauriInvoke('get_tv_keywords', { tmdbId })
            : await tauriInvoke('get_movie_keywords', { tmdbId });
        const list = document.getElementById('keywords-list');
        if (!list) return;
        if (!keywords || keywords.length === 0) {
            list.innerHTML = '<span style="color:var(--text2);font-size:13px">No keywords available.</span>';
            return;
        }
        list.innerHTML = keywords.map(k =>
            `<span style="background:var(--surface2);padding:4px 10px;border-radius:4px;font-size:12px">${escapeHtml(k)}</span>`
        ).join('');
    } catch (e) {
        console.error('Keywords error:', e);
    }
}

function renderStars(rating, playId, sourceType) {
    let s = '';
    for (let i = 1; i <= 10; i++) {
        const active = rating && i <= Math.round(rating) ? 'active' : '';
        s += `<span class="star ${active}" onclick="setRating(${playId}, ${i}, '${sourceType}')">★</span>`;
    }
    return s + (rating ? ` <span style="font-size:13px;color:var(--text2)">${rating.toFixed(1)}</span>` : '');
}

async function setRating(playId, rating, sourceType) {
    try {
        await tauriInvoke('update_rating', { playId, rating, sourceType });
        showToast('Rating updated!');
        if (currentDetail) showDetail(currentDetail.tmdbId, currentDetail.mediaType);
    } catch (e) {
        showToast('Error updating rating: ' + e);
    }
}

async function deletePlayEntry(playId, sourceType) {
    if (!confirm('Delete this play?')) return;
    try {
        await tauriInvoke('delete_play', { playId, sourceType });
        showToast('Play deleted');
        if (currentDetail) showDetail(currentDetail.tmdbId, currentDetail.mediaType);
    } catch (e) {
        showToast('Error: ' + e);
    }
}

async function toggleWatchlist(tmdbId, mediaType) {
    try {
        const inWl = await tauriInvoke('toggle_watchlist', { tmdbId, mediaType });
        showToast(inWl ? 'Added to watchlist' : 'Removed from watchlist');
        if (currentDetail) showDetail(currentDetail.tmdbId, currentDetail.mediaType);
    } catch (e) {
        showToast('Error: ' + e);
    }
}

async function addSeasonPlays() {
    const sel = document.getElementById('season-select');
    if (!sel || !sel.value) { showToast('Select a season'); return; }
    const seasonNumber = parseInt(sel.value);
    try {
        await tauriInvoke('add_season_plays', {
            input: { show_id: currentDetail.data.show.id, season_number: seasonNumber, watched_at: null, place_id: null, comment: null, user_rating: null }
        });
        showToast('Season plays logged!');
        if (currentDetail) showDetail(currentDetail.tmdbId, currentDetail.mediaType);
    } catch (e) {
        showToast('Error: ' + e);
    }
}

function showAddPlayDialog() {
    if (!currentDetail) return;
    const overlay = document.getElementById('modal-overlay');
    const content = document.getElementById('modal-content');
    const info = currentDetail.mediaType === 'tv' ? currentDetail.data.show : currentDetail.data.movie;
    const isTv = currentDetail.mediaType === 'tv';

    let episodeOptions = '';
    if (isTv && currentDetail.data.seasons) {
        // Build episode list
        const episodes = [];
        if (currentDetail.data.seasons) {
            for (const s of currentDetail.data.seasons) {
                for (let e = 1; e <= s.total_episodes; e++) {
                    episodes.push({ s: s.season_number, e, name: `S${String(s.season_number).padStart(2,'0')}E${String(e).padStart(2,'0')}` });
                }
            }
        }
        episodeOptions = episodes.map(ep =>
            `<option value="${ep.s}:${ep.e}">${ep.name}</option>`
        ).join('');
    }

    content.innerHTML = `
        <div class="modal-title">Add Play: ${escapeHtml(info.title || info.name)}</div>
        <div class="modal-form">
            <label>Watch Date</label>
            <input type="date" id="play-date" value="${new Date().toISOString().substring(0, 10)}">
            ${isTv ? `
            <label>Episode</label>
            <select id="play-episode">
                <option value="">Log all unwatched episodes</option>
                <option value="season:all">Log entire series</option>
                ${episodeOptions}
            </select>` : ''}
            <label>Rating (1-10, optional)</label>
            <input type="number" id="play-rating" min="1" max="10" step="0.5" placeholder="Rate after watching...">
            <label>Place (optional)</label>
            <select id="play-place"><option value="">None</option></select>
            <label>Comment (optional)</label>
            <textarea id="play-comment" placeholder="Notes..."></textarea>
            <div class="modal-actions">
                <button class="btn secondary" onclick="closeModal()">Cancel</button>
                <button class="btn primary" onclick="submitPlay()">Save Play</button>
            </div>
        </div>`;
    overlay.style.display = 'flex';

    // Load places
    tauriInvoke('get_places').then(places => {
        const sel = document.getElementById('play-place');
        if (sel) places.forEach(p => {
            sel.innerHTML += `<option value="${p.id}">${escapeHtml(p.name)}</option>`;
        });
    });
}

async function submitPlay() {
    if (!currentDetail) return;
    const info = currentDetail.mediaType === 'tv' ? currentDetail.data.show : currentDetail.data.movie;
    const watchedAt = document.getElementById('play-date').value;
    const rating = parseFloat(document.getElementById('play-rating').value) || null;
    const placeId = parseInt(document.getElementById('play-place').value) || null;
    const comment = document.getElementById('play-comment').value.trim() || null;
    const epSelect = document.getElementById('play-episode');
    const epVal = epSelect ? epSelect.value : '';

    try {
        if (currentDetail.mediaType === 'tv') {
            if (epVal && epVal.includes(':')) {
                const [sn, en] = epVal.split(':').map(Number);
                if (!isNaN(sn)) {
                    await tauriInvoke('add_season_plays', {
                        input: { show_id: info.id, season_number: sn, watched_at: watchedAt || null, place_id: placeId, comment, user_rating: rating }
                    });
                }
            } else {
                // Log season via selection or first season if available
                const firstSeason = currentDetail.data.seasons?.[0];
                if (firstSeason) {
                    await tauriInvoke('add_season_plays', {
                        input: { show_id: info.id, season_number: firstSeason.season_number, watched_at: watchedAt || null, place_id: placeId, comment, user_rating: rating }
                    });
                }
            }
        } else {
            await tauriInvoke('add_play', {
                input: { title_id: info.id, watched_at: watchedAt || null, place_id: placeId, comment, user_rating: rating }
            });
        }
        closeModal();
        showToast('Play saved!');
        if (currentDetail) showDetail(currentDetail.tmdbId, currentDetail.mediaType);
    } catch (e) {
        showToast('Error: ' + e);
    }
}

function closeModal() {
    document.getElementById('modal-overlay').style.display = 'none';
}

// === Watchlist ===
async function loadWatchlist() {
    try {
        const items = await tauriInvoke('get_watchlist');
        const grid = document.getElementById('watchlist-grid');
        if (!items || items.length === 0) {
            grid.innerHTML = '<p style="color:var(--text2);padding:20px;">Watchlist is empty.</p>';
            return;
        }
        grid.innerHTML = items.map(item => {
            const poster = item.poster ? `https://image.tmdb.org/t/p/w500${item.poster}` : null;
            const year = (item.release_date || '').substring(0, 4);
            const rating = item.tmdb_average ? `★ ${item.tmdb_average.toFixed(1)}` : '';
            return `<div class="card" onclick="showDetail(${item.tmdb_id}, '${item.media_type}')">
                <div class="card-poster">${poster ? `<img src="${poster}" alt="${escapeHtml(item.title)}" loading="lazy">` : '🎬'}</div>
                <div class="card-body">
                    <div class="card-title">${escapeHtml(item.title)}</div>
                    <div class="card-subtitle">${item.media_type === 'tv' ? 'TV' : 'Movie'} ${year ? '• ' + year : ''} ${rating}</div>
                </div>
            </div>`;
        }).join('');
    } catch (e) {
        console.error('Watchlist error:', e);
    }
}

// === Plays ===
async function loadAllPlays() {
    try {
        const plays = await tauriInvoke('get_all_plays');
        const grid = document.getElementById('plays-grid');
        if (!plays || plays.length === 0) {
            grid.innerHTML = '<p style="color:var(--text2);padding:20px;">No plays recorded.</p>';
            return;
        }
        grid.innerHTML = plays.map(p => createPlayCard(p)).join('');
    } catch (e) {
        console.error('Plays error:', e);
    }
}

function createPlayCard(p) {
    const poster = p.poster ? `https://image.tmdb.org/t/p/w500${p.poster}` : null;
    const d = p.watched_at ? new Date(p.watched_at + 'T00:00:00').toLocaleDateString() : 'No date';
    const ep = p.episode_id ? `S${String(p.season_number).padStart(2,'0')}E${String(p.episode_number).padStart(2,'0')}` : '';
    const type = p.media_type === 'tv' ? '📺' : '🎬';
    const rating = p.user_rating ? `★ ${p.user_rating.toFixed(1)}` : '';
    return `<div class="card" onclick="showDetail(${p.tmdb_id}, '${p.media_type}')">
        <div class="card-poster">${poster ? `<img src="${poster}" alt="${escapeHtml(p.title)}" loading="lazy">` : type}</div>
        ${rating ? `<div class="card-rating">${rating}</div>` : ''}
        <div class="card-body">
            <div class="card-title">${escapeHtml(p.title)}</div>
            <div class="card-subtitle">${d}${ep ? ' • ' + ep : ''}</div>
        </div>
    </div>`;
}

// === Favorites ===
document.querySelectorAll('.tab-bar .tab').forEach(tab => {
    tab.addEventListener('click', () => {
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
        tab.classList.add('active');
        document.getElementById('tab-' + tab.dataset.tab).classList.add('active');
        if (tab.dataset.tab === 'top-movies') loadFavoriteMovies();
        if (tab.dataset.tab === 'top-people') loadFavoritePeople('actor', 'favorite-people-grid');
        if (tab.dataset.tab === 'top-directors') loadFavoritePeople('director', 'favorite-directors-grid');
    });
});

async function loadFavorites() {
    await loadFavoriteMovies();
    await loadFavoritePeople('actor', 'favorite-people-grid');
    await loadFavoritePeople('director', 'favorite-directors-grid');
}

async function loadFavoriteMovies() {
    try {
        const items = await tauriInvoke('get_favorites', { minPlays: 1 });
        const grid = document.getElementById('favorites-grid');
        if (!items || items.length === 0) {
            grid.innerHTML = '<p style="color:var(--text2);padding:20px;">Rate movies and shows to see favorites here.</p>';
            return;
        }
        grid.innerHTML = items.map(item => {
            const poster = item.poster ? `https://image.tmdb.org/t/p/w500${item.poster}` : null;
            const year = (item.release_date || '').substring(0, 4);
            return `<div class="card" onclick="showDetail(${item.tmdb_id}, '${item.media_type}')">
                <div class="card-poster">${poster ? `<img src="${poster}" alt="${escapeHtml(item.title)}" loading="lazy">` : '🎬'}</div>
                <div class="card-rating">★ ${item.avg_rating.toFixed(1)}</div>
                <div class="card-body">
                    <div class="card-title">${escapeHtml(item.title)}</div>
                    <div class="card-subtitle">${year} • ${item.play_count} play${item.play_count > 1 ? 's' : ''}</div>
                </div>
            </div>`;
        }).join('');
    } catch (e) {
        console.error('Favorites error:', e);
    }
}

async function loadFavoritePeople(role, gridId) {
    try {
        const items = await tauriInvoke('get_favorite_people', { role, minPlays: 1 });
        const grid = document.getElementById(gridId);
        if (!items || items.length === 0) {
            grid.innerHTML = '<p style="color:var(--text2);padding:20px;">No frequent people found yet.</p>';
            return;
        }
        grid.innerHTML = items.map(p => {
            const avatar = p.profile_path ? `https://image.tmdb.org/t/p/w185${p.profile_path}` : null;
            return `<div class="person-card" onclick="viewPerson(${p.tmdb_person_id})">
                <div class="person-avatar">${avatar ? `<img src="${avatar}" alt="${escapeHtml(p.name)}">` : p.name.charAt(0)}</div>
                <div class="person-name">${escapeHtml(p.name)}</div>
                <div class="person-stat">★ ${p.avg_rating.toFixed(1)} • ${p.appearance_count} appearance${p.appearance_count > 1 ? 's' : ''}</div>
            </div>`;
        }).join('');
    } catch (e) {
        console.error('Favorite people error:', e);
    }
}

async function viewPerson(personId) {
    try {
        const data = await tauriInvoke('get_person_details', { personId });
        const d = data.details;
        const name = d.name || 'Unknown';
        const bio = d.biography || 'No biography available.';
        const avatar = d.profile_path ? `https://image.tmdb.org/t/p/w185${d.profile_path}` : null;
        const dept = d.known_for_department || '';
        const birth = d.birthday || '';
        const death = d.deathday || '';

        const overlay = document.getElementById('modal-overlay');
        const content = document.getElementById('modal-content');
        content.innerHTML = `
            <div class="modal-title" style="display:flex;align-items:center;gap:12px">
                ${avatar ? `<img src="${avatar}" style="width:48px;height:48px;border-radius:50%;object-fit:cover">` : `<div style="width:48px;height:48px;border-radius:50%;background:var(--accent2);display:flex;align-items:center;justify-content:center">${name.charAt(0)}</div>`}
                ${escapeHtml(name)}
            </div>
            ${dept ? `<div style="color:var(--text2);font-size:14px;margin-bottom:8px">${escapeHtml(dept)}</div>` : ''}
            ${birth ? `<div style="font-size:13px;color:var(--text2);margin-bottom:4px">Born: ${birth}${death ? ` • Died: ${death}` : ''}</div>` : ''}
            <div style="font-size:14px;line-height:1.6;margin:12px 0;max-height:300px;overflow-y:auto">${escapeHtml(bio)}</div>
            <div class="modal-actions">
                <button class="btn secondary" onclick="closeModal()">Close</button>
            </div>`;
        overlay.style.display = 'flex';
    } catch (e) {
        showToast('Error loading person: ' + e);
    }
}

// === Import / Export ===
document.getElementById('export-btn').addEventListener('click', async () => {
    try {
        const data = await tauriInvoke('export_data');
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `ratings-tracker-export-${new Date().toISOString().substring(0, 10)}.json`;
        a.click();
        URL.revokeObjectURL(url);
        showToast('Export downloaded!');
    } catch (e) {
        showToast('Export error: ' + e);
    }
});

document.getElementById('import-btn').addEventListener('click', () => {
    document.getElementById('import-file').click();
});

document.getElementById('import-file').addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;
    try {
        const text = await file.text();
        const data = JSON.parse(text);
        await tauriInvoke('import_data', { data });
        showToast('Import successful!');
        loadDashboard();
    } catch (e) {
        showToast('Import error: ' + e);
    }
});

// === Places ===
document.getElementById('add-place-btn').addEventListener('click', async () => {
    const name = document.getElementById('place-name').value.trim();
    if (!name) { showToast('Enter a place name'); return; }
    const isCinema = document.getElementById('place-cinema').checked;
    try {
        await tauriInvoke('add_place', { name, isCinema });
        document.getElementById('place-name').value = '';
        document.getElementById('place-cinema').checked = false;
        loadPlaces();
        showToast('Place added!');
    } catch (e) {
        showToast('Error: ' + e);
    }
});

async function loadPlaces() {
    try {
        const places = await tauriInvoke('get_places');
        const list = document.getElementById('places-list');
        list.innerHTML = places.map(p => `
            <div class="place-item">
                <div><span class="place-name">${escapeHtml(p.name)}</span>
                ${p.is_cinema ? `<span class="place-badge">Cinema</span>` : `<span class="place-badge">Home</span>`}</div>
                <button onclick="deletePlace(${p.id})">🗑</button>
            </div>
        `).join('');
    } catch (e) {
        console.error('Places error:', e);
    }
}

async function deletePlace(id) {
    if (!confirm('Delete this place?')) return;
    try {
        await tauriInvoke('delete_place', { id });
        loadPlaces();
        showToast('Place deleted');
    } catch (e) {
        showToast('Error: ' + e);
    }
}

// === Recommendations ===
document.getElementById('rec-btn').addEventListener('click', doRecommend);
document.getElementById('rec-prompt').addEventListener('keydown', e => { if (e.key === 'Enter') doRecommend(); });

async function initRecommendations() {
    const status = document.getElementById('rec-status');
    status.textContent = 'Get recommendations based on your top-rated favorites and TMDB.';
}

function useCurrentForRec() {
    if (!currentDetail) { showToast('Open a detail view first'); return; }
    const info = currentDetail.mediaType === 'tv' ? currentDetail.data.show : currentDetail.data.movie;
    const tmdbId = info.tmdb_id;
    const mediaType = currentDetail.mediaType;
    doRecommendForId(tmdbId, mediaType);
}

async function doRecommend() {
    const prompt = document.getElementById('rec-prompt').value.trim();
    if (!prompt) { showToast('Enter a prompt'); return; }
    doRecommendCommon({ prompt, referenceTmdbId: null, referenceMediaType: null });
}

async function doRecommendForId(tmdbId, mediaType) {
    doRecommendCommon({ prompt: '', referenceTmdbId: tmdbId, referenceMediaType: mediaType });
}

async function doRecommendCommon(params) {
    const resultsDiv = document.getElementById('rec-results');
    const statusDiv = document.getElementById('rec-status');
    resultsDiv.innerHTML = '<div class="loading"><div class="spinner"></div>Finding recommendations from TMDB...</div>';

    try {
        const results = await tauriInvoke('get_recommendations', params);
        if (!results || results.length === 0) {
            resultsDiv.innerHTML = '<p style="color:var(--text2);padding:20px;">No recommendations found from TMDB. Try a different prompt or rate more titles.</p>';
            statusDiv.textContent = '';
            return;
        }
        statusDiv.textContent = `Found ${results.length} recommendations from TMDB.`;
        resultsDiv.innerHTML = results.map(r => {
            const poster = r.poster ? `https://image.tmdb.org/t/p/w500${r.poster}` : null;
            const year = (r.release_date || '').substring(0, 4);
            const scorePct = Math.round(r.score * 100);
            const scoreColor = scorePct >= 70 ? 'var(--accent)' : scorePct >= 40 ? '#ffd700' : 'var(--text2)';
            return `<div class="card" onclick="showDetail(${r.tmdb_id}, '${r.media_type}')">
                <div class="card-poster">${poster ? `<img src="${poster}" alt="${escapeHtml(r.title)}" loading="lazy">` : '🎬'}</div>
                <div class="card-rating" style="color:${scoreColor}">${scorePct}%</div>
                <div class="card-body">
                    <div class="card-title">${escapeHtml(r.title)}</div>
                    <div class="card-subtitle">${year ? year + ' • ' : ''}${escapeHtml(r.match_reason)}</div>
                </div>
            </div>`;
        }).join('');
    } catch (e) {
        resultsDiv.innerHTML = `<p style="color:var(--accent);padding:20px;">Error: ${escapeHtml(e)}</p>`;
    }
}

// === Utilities ===
function escapeHtml(str) {
    if (!str) return '';
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

function showToast(msg) {
    const existing = document.querySelector('.toast');
    if (existing) existing.remove();
    const toast = document.createElement('div');
    toast.className = 'toast';
    toast.textContent = msg;
    document.body.appendChild(toast);
    setTimeout(() => toast.remove(), 3000);
}

// === Plays search ===
document.getElementById('plays-search')?.addEventListener('input', filterPlays);
document.getElementById('plays-sort')?.addEventListener('change', filterPlays);

function filterPlays() {
    const q = (document.getElementById('plays-search').value || '').toLowerCase();
    const sort = document.getElementById('plays-sort').value;
    const cards = document.querySelectorAll('#plays-grid .card');
    const entries = Array.from(cards);
    entries.forEach(c => {
        const title = c.querySelector('.card-title')?.textContent?.toLowerCase() || '';
        c.style.display = title.includes(q) ? '' : 'none';
    });
}

// === Init ===
document.addEventListener('DOMContentLoaded', () => {
    loadDashboard();
});

// Mock data for development outside Tauri
let mockDb = { plays: [], movies: [], watchlist: [] };
async function mockInvoke(cmd, args) {
    if (cmd === 'get_dashboard_stats') {
        return { movie_total_plays:0, tv_total_plays:0, movie_unique_titles:0, tv_unique_shows:0, movie_watchlist_count:0, tv_watchlist_count:0, total_runtime_minutes:0, total_plays:0 };
    }
    if (cmd === 'get_all_plays') return [];
    if (cmd === 'get_watchlist') return [];
    if (cmd === 'get_places') return [];
    if (cmd === 'get_favorites') return [];
    if (cmd === 'get_favorite_people') return [];
    if (cmd === 'get_recommendations') return [];
    if (cmd === 'search_titles') return [];
    if (cmd === 'export_data') return { movies:[], tv_shows:[], plays:[], tv_episode_plays:[], watchlist:[], tv_watchlist:[], places:[] };
    return null;
}
