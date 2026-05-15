import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid, Legend } from 'recharts'

interface ComparisonData {
  name: string
  goals: number
  assists: number
  points: number
  games: number
}

export default function ComparisonChart({ data }: { data: ComparisonData[] }) {
  if (!data.length) return <p className="text-sm text-slate-500">No data</p>

  return (
    <ResponsiveContainer width="100%" height={300}>
      <BarChart data={data}>
        <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
        <XAxis dataKey="name" stroke="#64748b" fontSize={12} />
        <YAxis stroke="#64748b" fontSize={12} />
        <Tooltip
          contentStyle={{ background: '#0f172a', border: '1px solid #1e293b', borderRadius: 8 }}
        />
        <Legend />
        <Bar dataKey="goals" fill="#22c55e" radius={[4, 4, 0, 0]} />
        <Bar dataKey="assists" fill="#a855f7" radius={[4, 4, 0, 0]} />
        <Bar dataKey="points" fill="#06b6d4" radius={[4, 4, 0, 0]} />
      </BarChart>
    </ResponsiveContainer>
  )
}
