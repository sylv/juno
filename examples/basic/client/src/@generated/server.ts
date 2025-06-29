/* eslint-disable */
/* tslint:disable */
/* Generated by Specta for Juno. DO NOT EDIT */
import { initTRPC } from '@trpc/server';

export type AddNumbersInput = { first: number; second: number }

export type GetUserInput = { user_id: number }

export type User = { id: number; name: string; nick: string | null }

const t = initTRPC.create();
const publicProcedure = t.procedure;
const appRouter = t.router({
    add_numbers: publicProcedure.input((value): AddNumbersInput => { throw new Error('Router should not be used') }).output((value): number => { throw new Error('Router should not be used') }).mutation((opts): number => { throw new Error('Router should not be used') }),
get_user: publicProcedure.input((value): GetUserInput => { throw new Error('Router should not be used') }).output((value): User => { throw new Error('Router should not be used') }).query((opts): User => { throw new Error('Router should not be used') }),
get_server_time: publicProcedure.output((value): string => { throw new Error('Router should not be used') }).query((opts): string => { throw new Error('Router should not be used') }),
get_api_version: publicProcedure.output((value): string => { throw new Error('Router should not be used') }).query((opts): string => { throw new Error('Router should not be used') }),
no_output: publicProcedure.query((opts): void => { throw new Error('Router should not be used') })
});

export type AppRouter = typeof appRouter;
