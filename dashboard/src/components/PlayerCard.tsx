import { Link } from 'react-router-dom'
import type { Player } from '../types'

const posColors: Record<string, string> = {
  C: 'bg-blue-600/20 text-blue-400',
  LW: 'bg-green-600/20 text-green-400',
  RW: 'bg-green-600/20 text-green-400',
  D: 'bg-purple-600/20 text-purple-400',
  G: 'bg-yellow-600/20 text-yellow-400',
}

export default function PlayerCard({ player }: { player: Player }) {
  return (
    <Link
      to={`/players/${player.nhl_player_id}`}
      className="card flex items-center gap-4 transition hover:border-slate-700 hover:bg-slate-800/50"
    >
      <div className="flex h-12 w-12 items-center justify-center rounded-full bg-slate-800 text-lg font-bold">
        {player.full_name.split(' ').map((w) => w[0]).join('').slice(0, 2)}
      </div>
      <div className="flex-1 min-w-0">
        <p className="truncate font-semibold">{player.full_name}</p>
        <p className="text-xs text-slate-500">
          {player.current_team_abbreviation || 'FA'} · #{player.jersey_number || '-'}
        </p>
      </div>
      <span className={`badge ${posColors[player.position] || 'badge-blue'}`}>
        {player.position}
      </span>
    </Link>
  )
}
