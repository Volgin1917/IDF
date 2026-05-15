import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from 'recharts'
import type { SeasonStats } from '../../types'

export default function PointsChart({ data }: { data: SeasonStats[] }) {
  const chartData = data
    .filter((s) => s.games_played > 0)
    .map((s) => ({
      season: s.season,
      points: s.points,
      goals: s.goals,
      assists: s.assists,
      games: s.games_played,
    }))
    .reverse()

  if (!chartData.length) return <p className="text-sm text-slate-500">No data</p>

  return (
    <ResponsiveContainer width="100%" height={280}>
      <LineChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
        <XAxis dataKey="season" stroke="#64748b" fontSize={12} />
        <YAxis stroke="#64748b" fontSize={12} />
        <Tooltip
          contentStyle={{ background: '#0f172a', border: '1px solid #1e293b', borderRadius: 8 }}
          labelStyle={{ color: '#e2e8f0' }}
        />
        <Line type="monotone" dataKey="points" stroke="#06b6d4" strokeWidth={2} dot={{ r: 4 }} />
        <Line type="monotone" dataKey="goals" stroke="#22c55e" strokeWidth={2} dot={{ r: 3 }} />
        <Line type="monotone" dataKey="assists" stroke="#a855f7" strokeWidth={2} dot={{ r: 3 }} />
      </LineChart>
    </ResponsiveContainer>
  )
}
