import { NavLink } from 'react-router-dom'
import {
  Home, Users, Building2, BarChart3, Newspaper, GitCompare,
} from 'lucide-react'

const links = [
  { to: '/', icon: Home, label: 'Overview' },
  { to: '/players', icon: Users, label: 'Players' },
  { to: '/teams', icon: Building2, label: 'Teams' },
  { to: '/analytics', icon: BarChart3, label: 'Analytics' },
  { to: '/news', icon: Newspaper, label: 'News' },
  { to: '/compare', icon: GitCompare, label: 'Compare' },
]

export default function Sidebar() {
  return (
    <aside className="flex h-full w-56 flex-col border-r border-slate-800 bg-slate-900">
      <div className="flex items-center gap-2 border-b border-slate-800 px-4 py-4">
        <div className="flex h-8 w-8 items-center justify-center rounded bg-ice-600 text-xs font-bold">
          IDF
        </div>
        <span className="text-sm font-semibold">ICE DATA FORGE</span>
      </div>
      <nav className="flex-1 space-y-1 p-3">
        {links.map((l) => (
          <NavLink
            key={l.to}
            to={l.to}
            end={l.to === '/'}
            className={({ isActive }) =>
              `flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors ${
                isActive
                  ? 'bg-ice-600/20 text-ice-400'
                  : 'text-slate-400 hover:bg-slate-800 hover:text-slate-200'
              }`
            }
          >
            <l.icon size={16} />
            {l.label}
          </NavLink>
        ))}
      </nav>
      <div className="border-t border-slate-800 px-4 py-3 text-xs text-slate-600">
        v0.1.0
      </div>
    </aside>
  )
}
