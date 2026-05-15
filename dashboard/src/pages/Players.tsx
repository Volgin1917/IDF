import { useEffect, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { api } from '../api/client'
import type { Player } from '../types'
import PlayerCard from '../components/PlayerCard'

export default function Players() {
  const [params] = useSearchParams()
  const query = params.get('q') || ''
  const [players, setPlayers] = useState<Player[]>([])
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (query.length < 2) return
    setLoading(true)
    api.searchPlayers(query, 30).then((r) => setPlayers(r.data)).catch(() => {}).finally(() => setLoading(false))
  }, [query])

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Players</h1>
      {!query && (
        <p className="text-sm text-slate-500">Use the search bar above or navigate from Overview</p>
      )}
      {query && <p className="text-sm text-slate-400">Results for: <span className="text-slate-200">'{query}'</span></p>}
      {loading && <p className="text-sm text-slate-500">Loading...</p>}
      {!loading && players.length === 0 && query.length >= 2 && (
        <p className="text-sm text-slate-500">No players found</p>
      )}
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {players.map((p) => (
          <PlayerCard key={p.nhl_player_id} player={p} />
        ))}
      </div>
    </div>
  )
}
