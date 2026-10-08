import { defineStore } from 'pinia'

const HISTORY_KEY = 'lulu:notifications'
const MAX_HISTORY = 30

export const useNotifyStore = defineStore('notify', {
    state: () => ({

        toasts: [],

        history: [],
        nextId: 1,
    }),

    getters: {
        unreadCount: (s) => s.history.filter(n => !n.read).length,

        orderedHistory: (s) => [...s.history].sort((a, b) => b.at - a.at),
    },

    actions: {

        hydrate() {
            if (!import.meta.client) return
            try {
                const raw = localStorage.getItem(HISTORY_KEY)
                const parsed = raw ? JSON.parse(raw) : []
                if (Array.isArray(parsed)) {
                    this.history = parsed.filter(n => n && typeof n.id === 'number')

                    this.nextId = this.history.reduce((m, n) => Math.max(m, n.id), 0) + 1
                }
            } catch {

                this.history = []
            }
        },

        // kind: 'success' | 'error' | 'info' | 'warning'
        push(kind, title, message = '', ms) {

            if (ms === undefined) ms = (kind === 'error' || kind === 'warning') ? 6000 : 4000
            const note = {
                id: this.nextId++,
                kind,
                title,
                message,
                at: Date.now(),
                read: false,
            }
            this.history.unshift(note)
            if (this.history.length > MAX_HISTORY) {
                this.history.length = MAX_HISTORY
            }
            this.persist()

            this.toasts.push(note)

            if (ms > 0) {
                setTimeout(() => this.dismiss(note.id), ms)
            }
            return note.id
        },

        success(title, message = '', ms) { return this.push('success', title, message, ms) },
        error(title, message = '', ms) { return this.push('error', title, message, ms) },
        info(title, message = '', ms) { return this.push('info', title, message, ms) },
        warning(title, message = '', ms) { return this.push('warning', title, message, ms) },

        dismiss(id) {
            const i = this.toasts.findIndex(t => t.id === id)
            if (i !== -1) this.toasts.splice(i, 1)
        },

        markAllRead() {
            for (const n of this.history) n.read = true
            this.persist()
        },

        markRead(id) {
            const n = this.history.find(x => x.id === id)
            if (n) {
                n.read = true
                this.persist()
            }
        },

        clearHistory() {
            this.history = []
            this.persist()
        },


        reset() {
            this.history = []
            this.toasts = []
            this.persist()
        },

        persist() {
            try {
                localStorage.setItem(HISTORY_KEY, JSON.stringify(this.history))
            } catch {
                // Private mode or a full quota must never break a page render.
            }
        },
    },
})
