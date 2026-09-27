//! MiniRust Leptos SSR application.

use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use leptos::prelude::*;
use minirust_core::APP_NAME;
use minirust_services::cqrs::QueryHandler;
use minirust_services::{GreetingQuery, GreetingQueryHandler};
use tower_http::trace::TraceLayer;

const CSS: &str = include_str!("generated.css");

/// Shared web application state.
#[derive(Clone)]
pub struct AppState {
    pub greeting: GreetingQueryHandler,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            greeting: GreetingQueryHandler,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
fn HomePage(message: String) -> impl IntoView {
    view! {
        <html lang="en" class="scroll-smooth bg-slate-950 text-slate-100">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="description" content="MiniRust is a Rust-first full-stack platform foundation built for long-term growth."/>
                <meta name="theme-color" content="#020617"/>
                <style>{CSS}</style>
                <title>{APP_NAME} {" - Rust-first platform foundation"}</title>
            </head>
            <body class="min-h-screen overflow-x-hidden bg-slate-950 antialiased">
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
                        <a href="#get-started" class="rounded-full border border-cyan-300/30 bg-cyan-300/10 px-4 py-2 text-sm font-semibold text-cyan-200 transition hover:bg-cyan-300/20 focus:outline-none focus:ring-2 focus:ring-cyan-400">Explore MiniRust</a>
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
                                    <a href="#capabilities" class="inline-flex items-center justify-center rounded-xl bg-cyan-300 px-5 py-3.5 text-sm font-bold text-slate-950 shadow-lg shadow-cyan-950/30 transition hover:bg-cyan-200 focus:outline-none focus:ring-2 focus:ring-cyan-300 focus:ring-offset-2 focus:ring-offset-slate-950">Explore the foundation <span class="ml-2">"->"</span></a>
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
            </body>
        </html>
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    let script = r#"
const form=document.querySelector('#login-form'),email=document.querySelector('#login-email'),code=document.querySelector('#login-code'),send=document.querySelector('#login-send'),verify=document.querySelector('#login-verify'),status=document.querySelector('#login-status');
const show=(m,b=false)=>{status.textContent=m;status.className=b?'mt-4 text-sm text-red-300':'mt-4 text-sm text-emerald-300'};
form.addEventListener('submit',async e=>{e.preventDefault();send.disabled=true;try{const r=await fetch('/api/v1/auth/login/request-code',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({email:email.value})});if(!r.ok){const p=await r.json();throw new Error(p.detail||'Unable to request code')}code.classList.remove('hidden');verify.classList.remove('hidden');send.classList.add('hidden');show('Verification code requested.')}catch(e){show(e.message,true)}finally{send.disabled=false}});
verify.addEventListener('click',async()=>{verify.disabled=true;try{const r=await fetch('/api/v1/auth/login/verify-code',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({email:email.value,code:code.value})});if(!r.ok){const p=await r.json();throw new Error(p.detail||'Invalid code')}location.href='/app'}catch(e){show(e.message,true)}finally{verify.disabled=false}});
"#;
    view! { <div class="mx-auto flex min-h-screen max-w-lg flex-col justify-center px-5 py-12">
        <a href="/" class="mb-8 text-sm font-bold text-cyan-300">&lt;- MiniRust</a>
        <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6 sm:p-8">
            <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">Sign in</p>
            <h1 class="mt-3 text-3xl font-black text-white">Access your account</h1>
            <p class="mt-3 text-sm leading-6 text-slate-400">Passwordless authentication uses a verification code.</p>
            <form id="login-form" class="mt-8 space-y-4">
                <label class="block text-sm font-semibold text-slate-200">Email<input id="login-email" type="email" required class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/></label>
                <label class="block text-sm font-semibold text-slate-200">Code<input id="login-code" type="text" inputmode="numeric" maxlength="6" class="mt-2 hidden w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/></label>
                <button id="login-send" type="submit" class="w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950">Send code</button>
                <button id="login-verify" type="button" class="hidden w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950">Verify and continue</button>
            </form>
            <p id="login-status" class="mt-4 text-sm text-slate-400"></p>
            <p class="mt-8 text-sm text-slate-500">New here? <a href="/register" class="font-semibold text-cyan-300">Create an account</a></p>
        </section>
    </div><script>{script}</script> }
}

#[component]
fn RegisterPage() -> impl IntoView {
    let script = r#"
const form=document.querySelector('#register-form'),email=document.querySelector('#register-email'),code=document.querySelector('#register-code'),send=document.querySelector('#register-send'),verify=document.querySelector('#register-verify'),status=document.querySelector('#register-status');
const show=(m,b=false)=>{status.textContent=m;status.className=b?'mt-4 text-sm text-red-300':'mt-4 text-sm text-emerald-300'};
form.addEventListener('submit',async e=>{e.preventDefault();send.disabled=true;try{const r=await fetch('/api/v1/auth/register/request-code',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({email:email.value})});if(!r.ok){const p=await r.json();throw new Error(p.detail||'Unable to request code')}code.classList.remove('hidden');verify.classList.remove('hidden');send.classList.add('hidden');show('Verification code requested.')}catch(e){show(e.message,true)}finally{send.disabled=false}});
verify.addEventListener('click',async()=>{verify.disabled=true;try{const r=await fetch('/api/v1/auth/register/verify-code',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({email:email.value,code:code.value})});if(!r.ok){const p=await r.json();throw new Error(p.detail||'Invalid code')}location.href='/app'}catch(e){show(e.message,true)}finally{verify.disabled=false}});
"#;
    view! { <div class="mx-auto flex min-h-screen max-w-lg flex-col justify-center px-5 py-12">
        <a href="/" class="mb-8 text-sm font-bold text-cyan-300">&lt;- MiniRust</a>
        <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-6 sm:p-8">
            <p class="text-sm font-bold uppercase tracking-widest text-cyan-300">Create account</p>
            <h1 class="mt-3 text-3xl font-black text-white">Start with your email</h1>
            <p class="mt-3 text-sm leading-6 text-slate-400">We will send a verification code.</p>
            <form id="register-form" class="mt-8 space-y-4">
                <label class="block text-sm font-semibold text-slate-200">Email<input id="register-email" type="email" required class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/></label>
                <label class="block text-sm font-semibold text-slate-200">Code<input id="register-code" type="text" inputmode="numeric" maxlength="6" class="mt-2 hidden w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/></label>
                <button id="register-send" type="submit" class="w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950">Send code</button>
                <button id="register-verify" type="button" class="hidden w-full rounded-xl bg-cyan-300 px-4 py-3 font-bold text-slate-950">Verify and continue</button>
            </form>
            <p id="register-status" class="mt-4 text-sm text-slate-400"></p>
            <p class="mt-8 text-sm text-slate-500">Already registered? <a href="/login" class="font-semibold text-cyan-300">Sign in</a></p>
        </section>
    </div><script>{script}</script> }
}

#[component]
fn AppPage() -> impl IntoView {
    let script = r#"
const status=document.querySelector('#profile-status'),form=document.querySelector('#profile-form'),email=document.querySelector('#profile-email'),name=document.querySelector('#profile-name'),avatar=document.querySelector('#profile-avatar'),admin=document.querySelector('#profile-admin'),logout=document.querySelector('#logout');
const show=(m,b=false)=>{status.textContent=m;status.className=b?'mt-4 text-sm text-red-300':'mt-4 text-sm text-emerald-300'};
(async()=>{const r=await fetch('/api/v1/auth/me');if(r.status===401){location.href='/login';return}if(!r.ok){show('Unable to load account.',true);return}const u=(await r.json()).data;email.value=u.email;name.value=u.full_name||'';avatar.value=u.avatar_url||'';admin.classList.toggle('hidden',!u.is_admin)})();
form.addEventListener('submit',async e=>{e.preventDefault();const r=await fetch('/api/v1/users/me',{method:'PATCH',headers:{'Content-Type':'application/json'},body:JSON.stringify({full_name:name.value||null,avatar_url:avatar.value||null})});if(r.ok)show('Profile updated.');else{const p=await r.json();show(p.detail||'Update failed.',true)}});
logout.addEventListener('click',async()=>{await fetch('/api/v1/auth/logout',{method:'POST'});location.href='/'});
"#;
    view! { <div class="min-h-screen"><header class="border-b border-white/10"><nav class="mx-auto flex max-w-7xl items-center justify-between px-5 py-5 sm:px-8 lg:px-10"><a href="/" class="font-black text-white">MiniRust</a><div class="flex gap-3"><a id="profile-admin" href="/admin" class="hidden rounded-xl border border-white/10 px-4 py-2 text-sm text-slate-300">Admin</a><button id="logout" class="rounded-xl bg-white/10 px-4 py-2 text-sm font-semibold text-white">Sign out</button></div></nav></header><main class="mx-auto max-w-7xl px-5 py-12 sm:px-8 lg:px-10"><p class="text-sm font-bold uppercase tracking-widest text-cyan-300">Account</p><h1 class="mt-3 text-4xl font-black text-white">Your workspace</h1><p class="mt-3 text-slate-400">Manage your profile and account session.</p><section class="mt-10 max-w-2xl rounded-3xl border border-white/10 bg-white/[0.03] p-6 sm:p-8"><form id="profile-form" class="space-y-5"><label class="block text-sm font-semibold text-slate-200">Email<input id="profile-email" readonly class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950/70 px-4 py-3 text-slate-400"/></label><label class="block text-sm font-semibold text-slate-200">Full name<input id="profile-name" class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/></label><label class="block text-sm font-semibold text-slate-200">Avatar URL<input id="profile-avatar" type="url" class="mt-2 w-full rounded-xl border border-white/10 bg-slate-950 px-4 py-3 text-white"/></label><button class="rounded-xl bg-cyan-300 px-5 py-3 font-bold text-slate-950">Save profile</button><p id="profile-status" class="text-sm text-slate-400"></p></form></section></main></div><script>{script}</script> }
}

#[component]
fn AdminPage() -> impl IntoView {
    let script = r#"
const status=document.querySelector('#admin-status'),rows=document.querySelector('#admin-rows');const show=(m,b=false)=>{status.textContent=m;status.className=b?'mt-3 text-sm text-red-300':'mt-3 text-sm text-slate-400'};const esc=s=>String(s??'').replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
(async()=>{const me=await fetch('/api/v1/auth/me');if(me.status===401){location.href='/login';return}if(!me.ok||!(await me.clone().json()).data.is_admin){location.href='/app';return}const r=await fetch('/api/v1/admin/users');if(!r.ok){show('Unable to load users.',true);return}const users=(await r.json()).data.users;rows.innerHTML=users.map(u=>'<tr class="border-t border-white/10"><td class="px-3 py-3 text-sm text-white">'+esc(u.email)+'</td><td class="px-3 py-3 text-sm text-slate-400">'+(u.is_admin?'Admin':'User')+'</td><td class="px-3 py-3 text-sm text-slate-400">'+(u.is_premium?'Premium':'—')+'</td><td class="px-3 py-3 text-right"><button data-email="'+esc(u.email)+'" class="unlock rounded-lg bg-white/10 px-3 py-1.5 text-xs font-semibold text-white">Unlock</button></td></tr>').join('');show(users.length+' users loaded.')})();
rows.addEventListener('click',async e=>{const b=e.target.closest('.unlock');if(!b)return;const r=await fetch('/api/v1/admin/users/'+encodeURIComponent(b.dataset.email)+'/unlock',{method:'POST'});if(r.ok){show('User unlocked.');location.reload()}else{const p=await r.json();show(p.detail||'Action failed.',true)}});
"#;
    view! { <div class="min-h-screen"><header class="border-b border-white/10"><nav class="mx-auto flex max-w-7xl items-center justify-between px-5 py-5 sm:px-8 lg:px-10"><a href="/app" class="font-black text-white">MiniRust</a><a href="/app" class="text-sm text-cyan-300">Back to workspace</a></nav></header><main class="mx-auto max-w-7xl px-5 py-12 sm:px-8 lg:px-10"><p class="text-sm font-bold uppercase tracking-widest text-cyan-300">Administration</p><h1 class="mt-3 text-4xl font-black text-white">Users</h1><p id="admin-status" class="mt-3 text-sm text-slate-400">Loading...</p><div class="mt-8 overflow-x-auto rounded-3xl border border-white/10 bg-white/[0.03]"><table class="w-full min-w-[680px] text-left"><thead><tr class="text-xs uppercase tracking-widest text-slate-500"><th class="px-3 py-4">Email</th><th class="px-3 py-4">Role</th><th class="px-3 py-4">Entitlement</th><th class="px-3 py-4 text-right">Action</th></tr></thead><tbody id="admin-rows"></tbody></table></div></main></div><script>{script}</script> }
}

/// Render the MiniRust landing page to an HTML string.
pub fn render_home_page(message: &str) -> String {
    let html = view! { <HomePage message=message.to_owned()/> }.to_html();
    format!("<!DOCTYPE html>{html}")
}

/// HTTP router used by the binary and by tests.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/app", get(app))
        .route("/admin", get(admin))
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn home(State(state): State<AppState>) -> impl IntoResponse {
    let greeting = state.greeting.handle(GreetingQuery);
    let html = render_home_page(&greeting.message);
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        )],
        html,
    )
}

async fn login() -> impl IntoResponse { (StatusCode::OK, [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))], format!("<!DOCTYPE html>{}", view! { <LoginPage/> }.to_html())) }
async fn register() -> impl IntoResponse { (StatusCode::OK, [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))], format!("<!DOCTYPE html>{}", view! { <RegisterPage/> }.to_html())) }
async fn app() -> impl IntoResponse { (StatusCode::OK, [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))], format!("<!DOCTYPE html>{}", view! { <AppPage/> }.to_html())) }
async fn admin() -> impl IntoResponse { (StatusCode::OK, [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))], format!("<!DOCTYPE html>{}", view! { <AdminPage/> }.to_html())) }

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[test]
    fn home_page_renders_the_landing_page() {
        let html = render_home_page("Hello from MiniRust");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("MiniRust<!> - Rust-first platform foundation"));
        assert!(html.contains("id=\"capabilities\""));
        assert!(html.contains("id=\"architecture\""));
        assert!(html.contains("Hello from MiniRust"));
        assert!(html.contains("md:flex"));
        assert!(html.contains("lg:grid-cols"));
    }

    #[tokio::test]
    async fn get_root_returns_responsive_landing_page_html() {
        let response = router(AppState::new())
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.starts_with("text/html"));

        let body = body_string(response).await;
        assert!(body.contains("MiniRust"));
        assert!(body.contains("viewport"));
        assert!(body.contains("@tailwindcss/browser@4"));
        assert!(body.contains("sm:text-6xl"));
        assert!(body.contains("md:flex"));
        assert!(body.contains("lg:grid-cols"));
    }

    #[tokio::test]
    async fn get_health_returns_ok() {
        let response = router(AppState::new())
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_string(response).await, "ok");
    }
}
