// This module defines utility types on top of the existing ones from TypeScript and type-fest.

export type ParametersObject<T extends (...args: any) => any> = {
    [K in Parameters<T>[number]]: Extract<Parameters<T>[number], K>;
};
