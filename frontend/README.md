# Sigurado frontend

SvelteKit 2 + Svelte 5 (runes) + Tailwind CSS v4 + shadcn-svelte.
Package manager: **pnpm**.

## Setup

```sh
pnpm install
```

## Developing

The frontend talks to the Rust backend, so start that first:

```sh
cd ../backend && cargo run --bin backend    # http://127.0.0.1:8080
```

Then:

```sh
pnpm dev

# or start the server and open the app in a new browser tab
pnpm dev --open
```

The API base URL defaults to `http://localhost:8080/api`. Override it with
`PUBLIC_API_BASE` in a `.env` file if the backend runs elsewhere.

## Checking

```sh
pnpm check        # svelte-check against the strict tsconfig
```

## Building

```sh
pnpm build
pnpm preview
```

> To deploy, you may need an [adapter](https://svelte.dev/docs/kit/adapters)
> for your target environment.
