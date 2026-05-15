import { useState } from 'react'
import { api } from '../api/client'
import type { Player, SeasonStats } from '../types'
import ComparisonChart from '../components/charts/ComparisonChart'

export default function Compare() {
  const [query, setQuery] = useState('')
  const [searchResults, setSearchResults] = useState<Player[]>([])
  const [selected, setSelected] = useState<Player[]>([])
  const [comparison, setComparison] = useState<{ player: Player; seasons: SeasonStats[] }[]>([])
  const [loading, setLoading] = useState(false)

  const search = async () => {
    if (query.length < 2) return
    try {
      const r = await api.searchPlayers(query, 10)
      setSearchResults(r.data.filter((p) => !selected.some((s) => s.nhl_player_id === p.nhl_player_id)))
    } catch { setSearchResults([]) }
  }

  const addPlayer = (p: Player) => {
    if (selected.length >= 5) return
    setSelected([...selected, p])
    setSearchResults([])
    setQuery('')
  }

  const removePlayer = (id: number) => {
    setSelected(selected.filter((p) => p.nhl_player_id !== id))
    setComparison([])
  }

  const runCompare = async () => {
    if (selected.length < 2) return
    setLoading(true)
    try {
      const r = await api.comparePlayers(selected.map((p) => p.nhl_player_id))
      setComparison(r.data.comparison)
    } catch { setComparison([]) }
    setLoading(false)
  }

  const chartData = comparison.map((c) => {
    const s = c.seasons[0]
    return {
      name: c.player.full_name,
      goals: s?.goals ?? 0,
      assists: s?.assists ?? 0,
      points: s?.points ?? 0,
      games: s?.games_played ?? 0,
    }
  })

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Compare Players</h1>

      <div className="flex gap-2">
        <input
          className="input flex-1"
          placeholder="Search players to add..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && search()}
        />
        <button onClick={search} className="btn-primary">Search</button>
      </div>

      {searchResults.length > 0 && (
        <div className="card space-y-1">
          {searchResults.map((p) => (
            <button
              key={p.nhl_player_id}
              onClick={() => addPlayer(p)}
              className="flex w-full items-center gap-3 rounded-md px-2 py-1.5 text-left text-sm hover:bg-slate-800"
            >
              <span className="font-medium">{p.full_name}</span>
              <span className="text-slate-500">{p.position} · {p.current_team_abbreviation || 'FA'}</span>
            </button>
          ))}
        </div>
      )}

      {selected.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {selected.map((p) => (
            <span key={p.nhl_player_id} className="badge-blue flex items-center gap-2 px-3 py-1.5">
              {p.full_name}
              <button onClick={() => removePlayer(p.nhl_player_id)} className="text-ice-400 hover:text-red-400">✕</button>
            </span>
          ))}
        </div>
      )}

      {selected.length >= 2 && (
        <button onClick={runCompare} disabled={loading} className="btn-primary">
          {loading ? 'Comparing...' : `Compare ${selected.length} Players`}
        </button>
      )}

      {comparison.length > 0 && (
        <>
          <div className="card">
            <h2 className="mb-3 font-semibold">Stat Comparison</h2>
            <ComparisonChart data={chartData} />
          </div>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {comparison.map((c) => {
              const s = c.seasons[0]
              return (
                <div key={c.player.nhl_player_id} className="card">
                  <p className="font-semibold">{c.player.full_name}</p>
                  {s ? (
                    <div className="mt-2 space-y-1 text-sm text-slate-400">
                      <p>GP: {s.games_played}</p>
                      <p>G: {s.goals} | A: {s.assists} | P: {s.points}</p>
                      <p>+/-: {s.plus_minus >= 0 ? '+' : ''}{s.plus_minus}</p>
                    </div>
                  ) : (
                    <p className="mt-2 text-sm text-slate-500">No stats</p>
                  )}
                </div>
              )
            })}
          </div>
        </>
      )}
    </div>
  )
}
