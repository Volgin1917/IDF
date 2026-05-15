const BASE = import.meta.env.VITE_API_BASE_URL || '/v1'

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`)
  if (!res.ok) throw new Error(`API ${res.status}: ${res.statusText}`)
  return res.json()
}

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new Error(`API ${res.status}: ${res.statusText}`)
  return res.json()
}

export interface LeaderQuery {
  metric: string
  season?: string
  position?: string
  limit?: number
  min_games?: number
}

export const api = {
  searchPlayers: (q: string, limit = 10) =>
    get<{ data: import('../types').Player[] }>(`/players/search?q=${encodeURIComponent(q)}&limit=${limit}`),

  getPlayer: (id: number) =>
    get<{ data: import('../types').Player }>(`/players/${id}`),

  getPlayerStats: (id: number, season?: string) => {
    const params = season ? `?season=${season}` : ''
    return get<{ data: { seasons: import('../types').SeasonStats[] } }>(`/players/${id}/stats${params}`)
  },

  getAiAnalysis: (id: number, type = 'full') =>
    get<{ analysis: import('../types').AiAnalysis }>(`/players/${id}/ai-analysis?type=${type}`),

  comparePlayers: (ids: number[], season?: string) =>
    post<{ data: { comparison: { player: import('../types').Player; seasons: import('../types').SeasonStats[] }[] } }>(
      '/players/compare', { player_ids: ids, season }
    ),

  getTeams: () =>
    get<{ data: import('../types').Team[] }>('/teams'),

  getTeamRoster: (abbr: string) =>
    get<{ data: import('../types').Player[] }>(`/teams/${abbr}/roster`),

  getNews: (limit = 20) =>
    get<{ data: import('../types').NewsEvent[] }>(`/news?limit=${limit}`),

  getLeaders: (q: LeaderQuery) => {
    const params = new URLSearchParams()
    Object.entries(q).forEach(([k, v]) => { if (v !== undefined) params.set(k, String(v)) })
    return get<{ data: { leaders: import('../types').LeaderEntry[]; metric: string; season: string } }>(
      `/analytics/leaders?${params}`
    )
  },

  getTimeline: (id: number) =>
    get<{ data: { player_id: number; seasons: import('../types').SeasonStats[] } }>(
      `/analytics/player/${id}/timeline`
    ),
}
