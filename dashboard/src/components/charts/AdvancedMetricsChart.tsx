import { RadarChart, PolarGrid, PolarAngleAxis, PolarRadiusAxis, Radar, ResponsiveContainer, Tooltip } from 'recharts'
import type { AdvancedMetrics } from '../../types'

export default function AdvancedMetricsChart({ metrics }: { metrics: AdvancedMetrics }) {
  const data = [
    { metric: 'PPG', value: (metrics.points_per_game ?? 0) / 2 * 100 },
    { metric: 'G/60', value: (metrics.goals_per_60 ?? 0) / 2 * 100 },
    { metric: 'A/60', value: (metrics.assists_per_60 ?? 0) / 2 * 100 },
    { metric: 'Corsi%', value: metrics.corsi_for_percentage ?? 50 },
    { metric: 'Fenwick%', value: metrics.fenwick_for_percentage ?? 50 },
    { metric: 'xG', value: (metrics.expected_goals ?? 0) * 10 },
    { metric: 'SH%', value: (metrics.shooting_percentage ?? 10) * 5 },
  ]

  return (
    <ResponsiveContainer width="100%" height={300}>
      <RadarChart data={data}>
        <PolarGrid stroke="#1e293b" />
        <PolarAngleAxis dataKey="metric" stroke="#64748b" fontSize={11} />
        <PolarRadiusAxis angle={30} domain={[0, 100]} stroke="#64748b" fontSize={10} />
        <Radar dataKey="value" stroke="#06b6d4" fill="#06b6d4" fillOpacity={0.2} />
        <Tooltip
          contentStyle={{ background: '#0f172a', border: '1px solid #1e293b', borderRadius: 8 }}
        />
      </RadarChart>
    </ResponsiveContainer>
  )
}
