import * as z from "zod";

export const ServiceStatusSchema = z.object({
    name: z.string(),
    healthy: z.boolean(),
    message: z.string().optional(),
});

export const BackendStatusSchema = z.object({
    version: z.string(),
    services: z.array(ServiceStatusSchema),
});

export class ServiceStatus {
    constructor(
        public readonly name: string,
        public readonly healthy: boolean,
        public readonly message?: string,
    ) {
    }

    static parse(json: unknown): z.infer<typeof ServiceStatusSchema> {
        return ServiceStatusSchema.parse(json);
    }

    static deserialize(json: z.infer<typeof ServiceStatusSchema>): ServiceStatus {
        return new ServiceStatus(json.name, json.healthy, json.message);
    }
}

export class BackendStatus {
    constructor(
        public readonly version: string,
        public readonly services: ServiceStatus[],
    ) {
    }

    get healthy(): boolean {
        return this.services.every(s => s.healthy);
    }

    static parse(json: unknown): z.infer<typeof BackendStatusSchema> {
        return BackendStatusSchema.parse(json);
    }

    static deserialize(json: z.infer<typeof BackendStatusSchema>): BackendStatus {
        return new BackendStatus(
            json.version,
            json.services.map(ServiceStatus.deserialize),
        );
    }
}
