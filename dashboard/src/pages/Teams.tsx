import { useEffect, useState } from 'react'
import { api } from '../api/client'
import type { Team } from '../types'

export default function Teams() {
  const [teams, setTeams] = useState<Team[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.getTeams().then((r) => setTeams(r.data)).catch(() => {}).finally(() => setLoading(false))
  }, [])

  const byConf = (conf: string) => teams.filter((t) => t.conference === conf && t.active)

  const ConferenceSection = ({ name }: { name: string }) => (
    <div>
      <h2 className="mb-3 text-lg font-semibold text-slate-400">{name}</h2>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {byConf(name).map((t) => (
          <div key={t.abbreviation} className="card">
            <p className="text-lg font-bold text-ice-400">{t.abbreviation}</p>
            <p className="text-sm font-medium">{t.location} {t.team_name}</p>
            <p className="text-xs text-slate-500">{t.division}</p>
          </div>
        ))}
      </div>
    </div>
  )

  if (loading) return <p className="text-slate-500">Loading teams...</p>

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-bold">NHL Teams</h1>
      <ConferenceSection name="Eastern" />
      <ConferenceSection name="Western" />
    </div>
  )
}
