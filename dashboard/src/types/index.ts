export interface Player {
  id: number
  nhl_player_id: number
  first_name: string
  last_name: string
  full_name: string
  position: string
  shoot_catches: string | null
  height_cm: number | null
  weight_lbs: number | null
  birth_date: string | null
  birth_city: string | null
  birth_country: string | null
  nationality: string | null
  current_team_abbreviation: string | null
  jersey_number: number | null
  is_active: boolean
}

export interface SeasonStats {
  season: string
  team: string | null
  games_played: number
  goals: number
  assists: number
  points: number
  plus_minus: number
  shots: number
  time_on_ice_per_game?: string | null
  advanced_metrics?: AdvancedMetrics | null
}

export interface AdvancedMetrics {
  points_per_game?: number
  goals_per_60?: number
  assists_per_60?: number
  points_per_60?: number
  shooting_percentage?: number
  corsi_for_percentage?: number
  fenwick_for_percentage?: number
  expected_goals?: number
  xg_per_60?: number
}

export interface Team {
  id: number
  nhl_team_id: number
  abbreviation: string
  team_name: string
  location: string
  conference: 'Eastern' | 'Western'
  division: string
  venue_name: string | null
  active: boolean
}

export interface NewsEvent {
  id: number
  title: string
  source: string
  url: string
  published_at: string
  category: string | null
  sentiment: string | null
  summary?: string | null
}

export interface AiAnalysis {
  summary?: string
  strengths?: string[]
  weaknesses?: string[]
  potential?: string | null
  confidence_score?: number
  model?: string
  cached?: boolean
  development_areas?: string[]
}

export interface LeaderEntry {
  rank: number
  player_id: number
  name: string
  team: string | null
  position: string
  value: number
  games: number
  advanced?: AdvancedMetrics | null
}

export interface ApiResponse<T> {
  data: T
}
