import {HttpErrorResponse} from "@angular/common/http";

export class DashboardHttpError implements Error {
    public name: string = "DashboardError";
    public message: string;
    public cause: HttpErrorResponse;

    constructor(
        error: HttpErrorResponse,
        public method: string,
    ) {
        this.message = error.message;
        this.cause = error;
    }

}
