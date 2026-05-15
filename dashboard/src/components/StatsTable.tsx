import type { SeasonStats } from '../types'

export default function StatsTable({ seasons }: { seasons: SeasonStats[] }) {
  if (!seasons.length) return <p className="text-sm text-slate-500">No stats</p>

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="border-b border-slate-800 text-slate-400">
            <th className="py-2 pr-4">Season</th>
            <th className="py-2 pr-4">Team</th>
            <th className="py-2 pr-4">GP</th>
            <th className="py-2 pr-4">G</th>
            <th className="py-2 pr-4">A</th>
            <th className="py-2 pr-4">P</th>
            <th className="py-2 pr-4">+/-</th>
            <th className="py-2 pr-4">S</th>
            <th className="py-2 pr-4">TOI</th>
          </tr>
        </thead>
        <tbody>
          {seasons.map((s) => (
            <tr key={s.season} className="border-b border-slate-800/50 hover:bg-slate-800/30">
              <td className="py-2 pr-4 font-medium">{s.season}</td>
              <td className="py-2 pr-4">{s.team || '-'}</td>
              <td className="py-2 pr-4">{s.games_played}</td>
              <td className="py-2 pr-4">{s.goals}</td>
              <td className="py-2 pr-4">{s.assists}</td>
              <td className="py-2 pr-4 font-semibold">{s.points}</td>
              <td className={`py-2 pr-4 ${(s.plus_minus ?? 0) >= 0 ? 'text-emerald-400' : 'text-red-400'}`}>
                {s.plus_minus >= 0 ? '+' : ''}{s.plus_minus}
              </td>
              <td className="py-2 pr-4">{s.shots}</td>
              <td className="py-2 pr-4 text-slate-500">{s.time_on_ice_per_game || '-'}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
