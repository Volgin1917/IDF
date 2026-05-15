import { useEffect, useState } from 'react'
import { api } from '../api/client'
import type { NewsEvent } from '../types'

const sentimentIcon: Record<string, string> = {
  positive: '🟢',
  negative: '🔴',
  neutral: '⚪',
}

export default function News() {
  const [news, setNews] = useState<NewsEvent[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.getNews(50).then((r) => setNews(r.data)).catch(() => {}).finally(() => setLoading(false))
  }, [])

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">NHL News</h1>
      {loading && <p className="text-sm text-slate-500">Loading...</p>}
      <div className="grid gap-3 lg:grid-cols-2">
        {news.map((n) => (
          <a key={n.id} href={n.url} target="_blank" rel="noopener noreferrer" className="card transition hover:border-slate-700">
            <div className="flex items-start gap-2">
              <span className="mt-0.5">{sentimentIcon[n.sentiment ?? ''] || '⚪'}</span>
              <div className="flex-1 min-w-0">
                <p className="font-medium">{n.title}</p>
                <p className="mt-1 text-xs text-slate-500">
                  {n.source} · {new Date(n.published_at).toLocaleDateString('en-US', {
                    year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
                  })}
                </p>
                {n.category && <span className="badge-blue mt-1 inline-block">{n.category}</span>}
              </div>
            </div>
          </a>
        ))}
      </div>
    </div>
  )
}
