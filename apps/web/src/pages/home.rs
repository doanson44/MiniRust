use leptos::prelude::*;
use minirust_core::APP_NAME;

#[component]
pub fn HomePage(message: String) -> impl IntoView {
    view! {
            <header class="border-b border-white/10 bg-slate-950/90 backdrop-blur">
                <nav class="mx-auto flex max-w-7xl items-center justify-between px-5 py-5 sm:px-8 lg:px-10" aria-label="Main navigation">
                    <a href="#top" class="flex items-center gap-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-cyan-400" aria-label="MiniRust home">
                        <span class="grid size-9 place-items-center rounded-xl bg-cyan-400 font-black text-slate-950">M</span>
                        <span class="text-lg font-bold tracking-tight">{APP_NAME}</span>
                    </a>
                    <div class="hidden items-center gap-8 text-sm text-slate-300 md:flex">
                        <a href="#capabilities" class="transition hover:text-white">Capabilities</a>
                        <a href="#architecture" class="transition hover:text-white">Architecture</a>
                        <a href="#principles" class="transition hover:text-white">Principles</a>
                    </div>
                    <div class="flex items-center gap-3"><a href="/login" class="rounded-full border border-white/10 px-4 py-2 text-sm font-semibold text-slate-300 transition hover:bg-white/10 hover:text-white focus:outline-none focus:ring-2 focus:ring-cyan-400">Sign in</a><a href="/register" class="rounded-full border border-cyan-300/30 bg-cyan-300/10 px-4 py-2 text-sm font-semibold text-cyan-200 transition hover:bg-cyan-300/20 focus:outline-none focus:ring-2 focus:ring-cyan-400">Get started</a></div>
                </nav>
            </header>

            <main id="top">
                <section class="relative isolate overflow-hidden">
                    <div class="absolute inset-x-0 top-0 -z-10 h-[32rem] bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.16),transparent_40%),radial-gradient(circle_at_top_left,rgba(59,130,246,0.14),transparent_35%)]"></div>
                    <div class="mx-auto grid max-w-7xl gap-14 px-5 pb-24 pt-20 sm:px-8 sm:pt-28 lg:grid-cols-[1.15fr_0.85fr] lg:items-center lg:px-10 lg:pb-32">
                        <div>
                            <div class="mb-7 inline-flex items-center gap-2 rounded-full border border-cyan-300/20 bg-cyan-300/5 px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.18em] text-cyan-200">
                                <span class="size-1.5 rounded-full bg-cyan-300"></span>
                                Rust-first full-stack foundation
                            </div>
                            <h1 class="max-w-4xl text-4xl font-black tracking-tight text-white sm:text-6xl lg:text-7xl">Build the platform once. <span class="text-cyan-300">Extend it for years.</span></h1>
                            <p class="mt-7 max-w-2xl text-base leading-8 text-slate-300 sm:text-lg">{message} MiniRust is structured as a reusable foundation for production-minded Rust applications, with SSR, REST APIs, CQRS-oriented services, persistence, testing, and containerized infrastructure.</p>
                            <div class="mt-9 flex flex-col gap-3 sm:flex-row">
                                <a href="/register" class="inline-flex items-center justify-center rounded-xl bg-cyan-300 px-5 py-3.5 text-sm font-bold text-slate-950 shadow-lg shadow-cyan-950/30 transition hover:bg-cyan-200 focus:outline-none focus:ring-2 focus:ring-cyan-300 focus:ring-offset-2 focus:ring-offset-slate-950">Create an account <span class="ml-2">"-> "</span></a>
                                <a href="#architecture" class="inline-flex items-center justify-center rounded-xl border border-white/15 bg-white/5 px-5 py-3.5 text-sm font-semibold text-white transition hover:bg-white/10 focus:outline-none focus:ring-2 focus:ring-white/50">See the architecture</a>
                            </div>
                            <div class="mt-10 grid max-w-xl grid-cols-1 gap-4 text-sm text-slate-400 sm:grid-cols-3">
                                <div><div class="font-bold text-white">Rust</div><div>Type-safe core</div></div>
                                <div><div class="font-bold text-white">SSR</div><div>Fast server rendering</div></div>
                                <div><div class="font-bold text-white">CQRS</div><div>Clear application boundaries</div></div>
                            </div>
                        </div>

                        <div class="relative mx-auto w-full max-w-xl">
                            <div class="absolute -inset-4 rounded-[2rem] bg-cyan-400/10 blur-3xl"></div>
                            <div class="relative overflow-hidden rounded-[1.75rem] border border-white/10 bg-white/[0.04] p-5 shadow-2xl shadow-black/30 sm:p-7">
                                <div class="flex items-center justify-between border-b border-white/10 pb-4">
                                    <div class="flex gap-1.5" aria-hidden="true"><span class="size-2.5 rounded-full bg-red-400"></span><span class="size-2.5 rounded-full bg-amber-400"></span><span class="size-2.5 rounded-full bg-emerald-400"></span></div>
                                    <span class="font-mono text-xs text-slate-500">minirust::platform</span>
                                </div>
                                <div class="space-y-4 pt-6 font-mono text-xs leading-6 sm:text-sm">
                                    <div class="text-slate-500">{"// one foundation, many capabilities"}</div>
                                    <div><span class="text-cyan-300">web</span> <span class="text-slate-500">"->"</span> <span class="text-white">SSR presentation</span></div>
                                    <div><span class="text-cyan-300">api</span> <span class="text-slate-500">"->"</span> <span class="text-white">REST transport</span></div>
                                    <div><span class="text-cyan-300">services</span> <span class="text-slate-500">"->"</span> <span class="text-white">commands + queries</span></div>
                                    <div><span class="text-cyan-300">database</span> <span class="text-slate-500">"->"</span> <span class="text-white">persistent state</span></div>
                                    <div><span class="text-cyan-300">tests</span> <span class="text-slate-500">"->"</span> <span class="text-white">real endpoint coverage</span></div>
                                    <div class="pt-2 text-emerald-300">status: foundation ready to evolve</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>

                <section id="capabilities" class="border-y border-white/10 bg-slate-900/50">
                    <div class="mx-auto max-w-7xl px-5 py-20 sm:px-8 lg:px-10 lg:py-24">
                        <div class="max-w-2xl">
                            <p class="text-sm font-bold uppercase tracking-[0.18em] text-cyan-300">Capabilities</p>
                            <h2 class="mt-3 text-3xl font-bold tracking-tight text-white sm:text-4xl">Designed to grow without losing structure.</h2>
                            <p class="mt-4 text-base leading-7 text-slate-400">The landing surface stays simple while the platform underneath can expand through well-defined application and infrastructure boundaries.</p>
                        </div>
                        <div class="mt-12 grid gap-5 md:grid-cols-2 lg:grid-cols-4">
                            <div class="rounded-2xl border border-white/10 bg-slate-950/70 p-6"><div class="text-2xl">01</div><h3 class="mt-5 font-bold text-white">Rust core</h3><p class="mt-3 text-sm leading-6 text-slate-400">Explicit ownership, strong types, and focused modules form the foundation for long-lived services.</p></div>
                            <div class="rounded-2xl border border-white/10 bg-slate-950/70 p-6"><div class="text-2xl">02</div><h3 class="mt-5 font-bold text-white">SSR web</h3><p class="mt-3 text-sm leading-6 text-slate-400">Leptos renders the web experience on the server and keeps presentation separate from business rules.</p></div>
                            <div class="rounded-2xl border border-white/10 bg-slate-950/70 p-6"><div class="text-2xl">03</div><h3 class="mt-5 font-bold text-white">CQRS services</h3><p class="mt-3 text-sm leading-6 text-slate-400">Commands and queries establish explicit application boundaries that can scale with feature count.</p></div>
                            <div class="rounded-2xl border border-white/10 bg-slate-950/70 p-6"><div class="text-2xl">04</div><h3 class="mt-5 font-bold text-white">Integration tests</h3><p class="mt-3 text-sm leading-6 text-slate-400">HTTP endpoints are exercised through the real router, with real infrastructure where required.</p></div>
                        </div>
                    </div>
                </section>

                <section id="architecture" class="mx-auto max-w-7xl px-5 py-20 sm:px-8 lg:px-10 lg:py-28">
                    <div class="grid gap-12 lg:grid-cols-[0.8fr_1.2fr] lg:items-center">
                        <div>
                            <p class="text-sm font-bold uppercase tracking-[0.18em] text-cyan-300">Architecture</p>
                            <h2 class="mt-3 text-3xl font-bold tracking-tight text-white sm:text-4xl">Keep responsibilities visible.</h2>
                            <p class="mt-5 text-base leading-7 text-slate-400">Presentation, transport, application services, domain logic, and infrastructure evolve independently while remaining connected through explicit contracts.</p>
                        </div>
                        <div class="rounded-3xl border border-white/10 bg-white/[0.03] p-5 sm:p-7">
                            <div class="space-y-3">
                                <div class="rounded-xl border border-cyan-300/20 bg-cyan-300/5 p-4"><div class="text-xs font-bold uppercase tracking-widest text-cyan-200">Presentation</div><div class="mt-1 font-semibold text-white">Leptos SSR</div></div>
                                <div class="mx-auto h-4 w-px bg-white/15"></div>
                                <div class="rounded-xl border border-white/10 bg-white/[0.03] p-4"><div class="text-xs font-bold uppercase tracking-widest text-slate-400">Transport</div><div class="mt-1 font-semibold text-white">Axum routes and handlers</div></div>
                                <div class="mx-auto h-4 w-px bg-white/15"></div>
                                <div class="grid gap-3 sm:grid-cols-2"><div class="rounded-xl border border-white/10 bg-white/[0.03] p-4"><div class="text-xs font-bold uppercase tracking-widest text-slate-400">Application</div><div class="mt-1 font-semibold text-white">Commands + Queries</div></div><div class="rounded-xl border border-white/10 bg-white/[0.03] p-4"><div class="text-xs font-bold uppercase tracking-widest text-slate-400">Domain</div><div class="mt-1 font-semibold text-white">Business rules</div></div></div>
                                <div class="mx-auto h-4 w-px bg-white/15"></div>
                                <div class="rounded-xl border border-white/10 bg-white/[0.03] p-4"><div class="text-xs font-bold uppercase tracking-widest text-slate-400">Infrastructure</div><div class="mt-1 font-semibold text-white">Repositories, database, external adapters</div></div>
                            </div>
                        </div>
                    </div>
                </section>

                <section id="principles" class="bg-cyan-300 text-slate-950">
                    <div class="mx-auto grid max-w-7xl gap-10 px-5 py-16 sm:px-8 lg:grid-cols-[1fr_auto] lg:items-center lg:px-10 lg:py-20">
                        <div><p class="text-sm font-bold uppercase tracking-[0.18em] text-slate-700">Engineering principles</p><h2 class="mt-3 max-w-3xl text-3xl font-black tracking-tight sm:text-4xl">Correctness first. Simplicity where possible. Explicit boundaries everywhere.</h2></div>
                        <div id="get-started" class="rounded-2xl bg-slate-950 px-6 py-5 text-white shadow-xl"><div class="text-sm font-bold">Ready to build?</div><div class="mt-1 text-sm text-slate-400">Start with the next vertical slice.</div></div>
                    </div>
                </section>
            </main>

            <footer class="border-t border-white/10 bg-slate-950">
                <div class="mx-auto flex max-w-7xl flex-col gap-3 px-5 py-8 text-sm text-slate-500 sm:px-8 md:flex-row md:items-center md:justify-between lg:px-10">
                    <div><span class="font-semibold text-slate-300">{APP_NAME}</span> <span class="mx-2">"."</span> Rust-first platform foundation</div>
                    <div>Built with Rust, Axum, Leptos, and a CQRS-oriented service layer.</div>
                </div>
            </footer>
    }
}
