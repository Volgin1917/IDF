import { useEffect, useState } from 'react'
import { api } from '../api/client'
import type { LeaderEntry } from '../types'
import { Link } from 'react-router-dom'

const METRICS = [
  { key: 'points', label: 'Points' },
  { key: 'goals', label: 'Goals' },
  { key: 'assists', label: 'Assists' },
  { key: 'plus_minus', label: '+/-' },
  { key: 'shots', label: 'Shots' },
  { key: 'points_per_game', label: 'PPG' },
  { key: 'shooting_percentage', label: 'S%' },
]

export default function Analytics() {
  const [metric, setMetric] = useState('points')
  const [leaders, setLeaders] = useState<LeaderEntry[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    setLoading(true)
    api.getLeaders({ metric, limit: 20 })
      .then((r) => setLeaders(r.data.leaders))
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [metric])

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Analytics</h1>

      <div className="flex flex-wrap gap-2">
        {METRICS.map((m) => (
          <button
            key={m.key}
            onClick={() => setMetric(m.key)}
            className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
              metric === m.key
                ? 'bg-ice-600 text-white'
                : 'bg-slate-800 text-slate-400 hover:bg-slate-700'
            }`}
          >
            {m.label}
          </button>
        ))}
      </div>

      {loading ? (
        <p className="text-sm text-slate-500">Loading...</p>
      ) : (
        <div className="card">
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead>
                <tr className="border-b border-slate-800 text-slate-400">
                  <th className="py-2 pr-4 w-8">#</th>
                  <th className="py-2 pr-4">Player</th>
                  <th className="py-2 pr-4">Team</th>
                  <th className="py-2 pr-4">Pos</th>
                  <th className="py-2 pr-4">GP</th>
                  <th className="py-2 pr-4 font-bold">{METRICS.find((m) => m.key === metric)?.label || metric}</th>
                </tr>
              </thead>
              <tbody>
                {leaders.map((l) => (
                  <tr key={l.player_id} className="border-b border-slate-800/50 hover:bg-slate-800/30">
                    <td className="py-2 pr-4 text-slate-500">{l.rank}</td>
                    <td className="py-2 pr-4">
                      <Link to={`/players/${l.player_id}`} className="font-medium hover:text-ice-400">
                        {l.name}
                      </Link>
                    </td>
                    <td className="py-2 pr-4">{l.team || '-'}</td>
                    <td className="py-2 pr-4">{l.position}</td>
                    <td className="py-2 pr-4">{l.games}</td>
                    <td className="py-2 pr-4 font-semibold text-ice-400">{l.value}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  )
}
