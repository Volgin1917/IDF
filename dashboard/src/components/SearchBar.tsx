import { useState, type FormEvent } from 'react'
import { Search } from 'lucide-react'
import { useNavigate } from 'react-router-dom'

export default function SearchBar() {
  const [q, setQ] = useState('')
  const nav = useNavigate()

  const submit = (e: FormEvent) => {
    e.preventDefault()
    if (q.trim().length >= 2) nav(`/players?q=${encodeURIComponent(q.trim())}`)
  }

  return (
    <form onSubmit={submit} className="relative w-full max-w-md">
      <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
      <input
        className="input pl-9"
        placeholder="Search players..."
        value={q}
        onChange={(e) => setQ(e.target.value)}
      />
    </form>
  )
}
