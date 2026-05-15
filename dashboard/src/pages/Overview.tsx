import { useEffect, useState } from 'react'
import { TrendingUp, Users, Building2, Newspaper } from 'lucide-react'
import { api } from '../api/client'
import type { LeaderEntry, NewsEvent } from '../types'
import SearchBar from '../components/SearchBar'
import { Link } from 'react-router-dom'

export default function Overview() {
  const [leaders, setLeaders] = useState<LeaderEntry[]>([])
  const [news, setNews] = useState<NewsEvent[]>([])

  useEffect(() => {
    api.getLeaders({ metric: 'points', limit: 5 }).then((r) => setLeaders(r.data.leaders)).catch(() => {})
    api.getNews(5).then((r) => setNews(r.data)).catch(() => {})
  }, [])

  const stats = [
    { icon: TrendingUp, label: 'Players Tracked', value: '1,200+' },
    { icon: Users, label: 'Active Players', value: '800+' },
    { icon: Building2, label: 'Teams', value: '32' },
    { icon: Newspaper, label: 'News Items', value: '5,000+' },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">ICE DATA FORGE</h1>
        <p className="mt-1 text-sm text-slate-500">NHL Analytics Platform</p>
      </div>

      <SearchBar />

      <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        {stats.map((s) => (
          <div key={s.label} className="card">
            <div className="flex items-center gap-3">
              <s.icon size={20} className="text-ice-500" />
              <div>
                <p className="text-2xl font-bold">{s.value}</p>
                <p className="text-xs text-slate-500">{s.label}</p>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <div className="card">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="font-semibold">Points Leaders</h2>
            <Link to="/analytics" className="text-xs text-ice-500 hover:underline">View all</Link>
          </div>
          <div className="space-y-2">
            {leaders.map((l) => (
              <Link
                key={l.player_id}
                to={`/players/${l.player_id}`}
                className="flex items-center justify-between rounded-md px-2 py-1.5 hover:bg-slate-800"
              >
                <div className="flex items-center gap-3">
                  <span className="w-5 text-center text-sm font-bold text-slate-500">#{l.rank}</span>
                  <div>
                    <p className="text-sm font-medium">{l.name}</p>
                    <p className="text-xs text-slate-500">{l.team} · {l.position}</p>
                  </div>
                </div>
                <span className="font-bold text-ice-400">{l.value}</span>
              </Link>
            ))}
          </div>
        </div>

        <div className="card">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="font-semibold">Latest News</h2>
            <Link to="/news" className="text-xs text-ice-500 hover:underline">View all</Link>
          </div>
          <div className="space-y-2">
            {news.map((n) => (
              <a key={n.id} href={n.url} target="_blank" rel="noopener noreferrer"
                className="block rounded-md px-2 py-1.5 hover:bg-slate-800">
                <p className="truncate text-sm">{n.title}</p>
                <p className="text-xs text-slate-500">{n.source} · {new Date(n.published_at).toLocaleDateString()}</p>
              </a>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
