<template>
    <div class="hud-frame glass rounded-[1.5rem] border border-emerald-100 p-8 relative overflow-hidden md:p-10 text-center tech-pattern">
        <span class="hud-corner hud-corner-tl"></span><span class="hud-corner hud-corner-br"></span>
        <div class="pointer-events-none absolute inset-0 tech-dots opacity-[0.04]"></div>
        <div class="w-16 h-16 mx-auto rounded-2xl bg-gradient-to-br from-emerald-500 to-green-600 flex items-center justify-center shadow-lg shadow-emerald-200">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z"/><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/></svg>
        </div>
        <h2 class="display text-xl font-extrabold text-green-900 mt-4">General Settings</h2>
        <p class="mono text-[11px] tracking-[0.16em] text-emerald-700/50 mt-1">CONFIGURATION</p>
        <p class="text-sm text-emerald-700/60 mt-2 max-w-md mx-auto">General system configuration — secure & realtime. Full features are on the way.</p>
        <div class="mt-6 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-amber-100 text-amber-800 text-xs font-bold border border-amber-200">🚧 Coming soon</div>
    </div>
</template>
