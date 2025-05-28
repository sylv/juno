import { createTRPCClient, httpLink } from "@trpc/client";
import type { AppRouter } from "./@generated/server";

const client = createTRPCClient<AppRouter>({
	links: [
		httpLink({
			url: "http://localhost:3000/trpc",
		}),
	],
});

const user = await client.get_user.query({ user_id: 2 });
console.log({ user });

const serverTime = await client.get_server_time.query();
console.log({ serverTime });

const apiVersion = await client.get_api_version.query();
console.log({ apiVersion });

const sum = await client.add_numbers.mutate({ first: 1, second: 2 });
console.log({ sum });

await client.no_output.query();
