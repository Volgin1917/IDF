import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { api } from '../api/client'
import type { Player, SeasonStats, AiAnalysis } from '../types'
import StatsTable from '../components/StatsTable'
import PointsChart from '../components/charts/PointsChart'
import AdvancedMetricsChart from '../components/charts/AdvancedMetricsChart'

export default function PlayerDetail() {
  const { id } = useParams<{ id: string }>()
  const [player, setPlayer] = useState<Player | null>(null)
  const [stats, setStats] = useState<SeasonStats[]>([])
  const [analysis, setAnalysis] = useState<AiAnalysis | null>(null)
  const [tab, setTab] = useState<'stats' | 'analysis'>('stats')

  useEffect(() => {
    if (!id) return
    const pid = Number(id)
    api.getPlayer(pid).then((r) => setPlayer(r.data)).catch(() => {})
    api.getPlayerStats(pid).then((r) => setStats(r.data.seasons)).catch(() => {})
    api.getAiAnalysis(pid).then((r) => setAnalysis(r.analysis)).catch(() => {})
  }, [id])

  if (!player) return <p className="text-slate-500">Loading...</p>

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <div className="flex h-16 w-16 items-center justify-center rounded-full bg-slate-800 text-2xl font-bold">
          {player.full_name.split(' ').map((w) => w[0]).join('')}
        </div>
        <div>
          <h1 className="text-2xl font-bold">{player.full_name}</h1>
          <p className="text-sm text-slate-500">
            {player.current_team_abbreviation || 'FA'} · {player.position}
            {player.jersey_number ? ` · #${player.jersey_number}` : ''}
          </p>
          <div className="mt-1 flex gap-2 text-xs text-slate-600">
            {player.height_cm && <span>{player.height_cm} cm</span>}
            {player.weight_lbs && <span>{player.weight_lbs} lbs</span>}
            {player.birth_country && <span>{player.birth_country}</span>}
          </div>
        </div>
      </div>

      <div className="flex gap-1 rounded-lg bg-slate-900 p-1">
        {(['stats', 'analysis'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`flex-1 rounded-md px-4 py-2 text-sm font-medium transition ${
              tab === t ? 'bg-ice-600 text-white' : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            {t === 'stats' ? 'Stats' : 'AI Analysis'}
          </button>
        ))}
      </div>

      {tab === 'stats' && (
        <div className="grid gap-6 lg:grid-cols-2">
          <div className="card">
            <h2 className="mb-3 font-semibold">Career Stats</h2>
            <StatsTable seasons={stats} />
          </div>
          <div className="card">
            <h2 className="mb-3 font-semibold">Points Timeline</h2>
            <PointsChart data={stats} />
          </div>
          {stats[0]?.advanced_metrics && (
            <div className="card lg:col-span-2">
              <h2 className="mb-3 font-semibold">Advanced Metrics</h2>
              <AdvancedMetricsChart metrics={stats[0].advanced_metrics} />
            </div>
          )}
        </div>
      )}

      {tab === 'analysis' && (
        <div className="card space-y-4">
          {analysis ? (
            <>
              {analysis.summary && <p className="text-sm leading-relaxed text-slate-300">{analysis.summary}</p>}
              <div className="grid gap-4 sm:grid-cols-2">
                <div>
                  <h3 className="mb-2 text-sm font-semibold text-emerald-400">Strengths</h3>
                  <ul className="space-y-1">
                    {(analysis.strengths ?? []).map((s, i) => (
                      <li key={i} className="text-sm text-slate-300">• {s}</li>
                    ))}
                  </ul>
                </div>
                <div>
                  <h3 className="mb-2 text-sm font-semibold text-red-400">Weaknesses</h3>
                  <ul className="space-y-1">
                    {(analysis.weaknesses ?? []).map((w, i) => (
                      <li key={i} className="text-sm text-slate-300">• {w}</li>
                    ))}
                  </ul>
                </div>
              </div>
              {analysis.potential && (
                <div>
                  <h3 className="mb-1 text-sm font-semibold text-ice-400">Potential</h3>
                  <p className="text-sm text-slate-300">{analysis.potential}</p>
                </div>
              )}
              <div className="flex gap-4 text-xs text-slate-600">
                {analysis.confidence_score != null && (
                  <span>Confidence: {(analysis.confidence_score * 100).toFixed(0)}%</span>
                )}
                {analysis.model && <span>Model: {analysis.model}</span>}
                {analysis.cached && <span>Cached</span>}
              </div>
            </>
          ) : (
            <p className="text-sm text-slate-500">No AI analysis available yet</p>
          )}
        </div>
      )}
    </div>
  )
}
