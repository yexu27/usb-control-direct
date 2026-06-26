import * as $protobuf from "protobufjs";
import Long = require("long");
/** Namespace usb_control. */
export namespace usb_control {

    /** Properties of a CmdLogin. */
    interface ICmdLogin {

        /** CmdLogin username */
        username?: (string|null);

        /** CmdLogin password */
        password?: (string|null);
    }

    /** Represents a CmdLogin. */
    class CmdLogin implements ICmdLogin {

        /**
         * Constructs a new CmdLogin.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdLogin);

        /** CmdLogin username. */
        public username: string;

        /** CmdLogin password. */
        public password: string;

        /**
         * Encodes the specified CmdLogin message. Does not implicitly {@link usb_control.CmdLogin.verify|verify} messages.
         * @param message CmdLogin message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdLogin, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdLogin message, length delimited. Does not implicitly {@link usb_control.CmdLogin.verify|verify} messages.
         * @param message CmdLogin message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdLogin, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdLogin message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdLogin
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdLogin;

        /**
         * Decodes a CmdLogin message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdLogin
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdLogin;

        /**
         * Creates a CmdLogin message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdLogin
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdLogin;

        /**
         * Creates a plain object from a CmdLogin message. Also converts values to other types if specified.
         * @param message CmdLogin
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdLogin, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdLogin to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdLogin
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspLogin. */
    interface IRspLogin {

        /** RspLogin success */
        success?: (boolean|null);

        /** RspLogin sessionToken */
        sessionToken?: (string|null);

        /** RspLogin username */
        username?: (string|null);

        /** RspLogin role */
        role?: (string|null);

        /** RspLogin authorized */
        authorized?: (boolean|null);

        /** RspLogin authExpireTime */
        authExpireTime?: (number|Long|null);

        /** RspLogin deviceDescription */
        deviceDescription?: (string|null);

        /** RspLogin resultCode */
        resultCode?: (number|null);

        /** RspLogin errorMessage */
        errorMessage?: (string|null);

        /** RspLogin authStatus */
        authStatus?: (string|null);
    }

    /** Represents a RspLogin. */
    class RspLogin implements IRspLogin {

        /**
         * Constructs a new RspLogin.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspLogin);

        /** RspLogin success. */
        public success: boolean;

        /** RspLogin sessionToken. */
        public sessionToken: string;

        /** RspLogin username. */
        public username: string;

        /** RspLogin role. */
        public role: string;

        /** RspLogin authorized. */
        public authorized: boolean;

        /** RspLogin authExpireTime. */
        public authExpireTime: (number|Long);

        /** RspLogin deviceDescription. */
        public deviceDescription: string;

        /** RspLogin resultCode. */
        public resultCode: number;

        /** RspLogin errorMessage. */
        public errorMessage: string;

        /** RspLogin authStatus. */
        public authStatus: string;

        /**
         * Encodes the specified RspLogin message. Does not implicitly {@link usb_control.RspLogin.verify|verify} messages.
         * @param message RspLogin message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspLogin, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspLogin message, length delimited. Does not implicitly {@link usb_control.RspLogin.verify|verify} messages.
         * @param message RspLogin message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspLogin, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspLogin message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspLogin
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspLogin;

        /**
         * Decodes a RspLogin message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspLogin
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspLogin;

        /**
         * Creates a RspLogin message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspLogin
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspLogin;

        /**
         * Creates a plain object from a RspLogin message. Also converts values to other types if specified.
         * @param message RspLogin
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspLogin, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspLogin to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspLogin
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdAuthStatusQuery. */
    interface ICmdAuthStatusQuery {

        /** CmdAuthStatusQuery sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdAuthStatusQuery. */
    class CmdAuthStatusQuery implements ICmdAuthStatusQuery {

        /**
         * Constructs a new CmdAuthStatusQuery.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdAuthStatusQuery);

        /** CmdAuthStatusQuery sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdAuthStatusQuery message. Does not implicitly {@link usb_control.CmdAuthStatusQuery.verify|verify} messages.
         * @param message CmdAuthStatusQuery message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdAuthStatusQuery, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdAuthStatusQuery message, length delimited. Does not implicitly {@link usb_control.CmdAuthStatusQuery.verify|verify} messages.
         * @param message CmdAuthStatusQuery message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdAuthStatusQuery, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdAuthStatusQuery message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdAuthStatusQuery
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdAuthStatusQuery;

        /**
         * Decodes a CmdAuthStatusQuery message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdAuthStatusQuery
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdAuthStatusQuery;

        /**
         * Creates a CmdAuthStatusQuery message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdAuthStatusQuery
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdAuthStatusQuery;

        /**
         * Creates a plain object from a CmdAuthStatusQuery message. Also converts values to other types if specified.
         * @param message CmdAuthStatusQuery
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdAuthStatusQuery, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdAuthStatusQuery to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdAuthStatusQuery
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspAuthStatus. */
    interface IRspAuthStatus {

        /** RspAuthStatus authorized */
        authorized?: (boolean|null);

        /** RspAuthStatus expireTime */
        expireTime?: (number|Long|null);

        /** RspAuthStatus deviceDescription */
        deviceDescription?: (string|null);

        /** RspAuthStatus authStatus */
        authStatus?: (string|null);
    }

    /** Represents a RspAuthStatus. */
    class RspAuthStatus implements IRspAuthStatus {

        /**
         * Constructs a new RspAuthStatus.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspAuthStatus);

        /** RspAuthStatus authorized. */
        public authorized: boolean;

        /** RspAuthStatus expireTime. */
        public expireTime: (number|Long);

        /** RspAuthStatus deviceDescription. */
        public deviceDescription: string;

        /** RspAuthStatus authStatus. */
        public authStatus: string;

        /**
         * Encodes the specified RspAuthStatus message. Does not implicitly {@link usb_control.RspAuthStatus.verify|verify} messages.
         * @param message RspAuthStatus message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspAuthStatus, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspAuthStatus message, length delimited. Does not implicitly {@link usb_control.RspAuthStatus.verify|verify} messages.
         * @param message RspAuthStatus message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspAuthStatus, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspAuthStatus message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspAuthStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspAuthStatus;

        /**
         * Decodes a RspAuthStatus message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspAuthStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspAuthStatus;

        /**
         * Creates a RspAuthStatus message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspAuthStatus
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspAuthStatus;

        /**
         * Creates a plain object from a RspAuthStatus message. Also converts values to other types if specified.
         * @param message RspAuthStatus
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspAuthStatus, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspAuthStatus to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspAuthStatus
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdGetMachineCode. */
    interface ICmdGetMachineCode {

        /** CmdGetMachineCode sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdGetMachineCode. */
    class CmdGetMachineCode implements ICmdGetMachineCode {

        /**
         * Constructs a new CmdGetMachineCode.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdGetMachineCode);

        /** CmdGetMachineCode sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdGetMachineCode message. Does not implicitly {@link usb_control.CmdGetMachineCode.verify|verify} messages.
         * @param message CmdGetMachineCode message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdGetMachineCode, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdGetMachineCode message, length delimited. Does not implicitly {@link usb_control.CmdGetMachineCode.verify|verify} messages.
         * @param message CmdGetMachineCode message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdGetMachineCode, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdGetMachineCode message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdGetMachineCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdGetMachineCode;

        /**
         * Decodes a CmdGetMachineCode message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdGetMachineCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdGetMachineCode;

        /**
         * Creates a CmdGetMachineCode message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdGetMachineCode
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdGetMachineCode;

        /**
         * Creates a plain object from a CmdGetMachineCode message. Also converts values to other types if specified.
         * @param message CmdGetMachineCode
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdGetMachineCode, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdGetMachineCode to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdGetMachineCode
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspMachineCode. */
    interface IRspMachineCode {

        /** RspMachineCode machineCode */
        machineCode?: (string|null);

        /** RspMachineCode qrcodePng */
        qrcodePng?: (Uint8Array|null);
    }

    /** Represents a RspMachineCode. */
    class RspMachineCode implements IRspMachineCode {

        /**
         * Constructs a new RspMachineCode.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspMachineCode);

        /** RspMachineCode machineCode. */
        public machineCode: string;

        /** RspMachineCode qrcodePng. */
        public qrcodePng: Uint8Array;

        /**
         * Encodes the specified RspMachineCode message. Does not implicitly {@link usb_control.RspMachineCode.verify|verify} messages.
         * @param message RspMachineCode message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspMachineCode, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspMachineCode message, length delimited. Does not implicitly {@link usb_control.RspMachineCode.verify|verify} messages.
         * @param message RspMachineCode message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspMachineCode, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspMachineCode message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspMachineCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspMachineCode;

        /**
         * Decodes a RspMachineCode message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspMachineCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspMachineCode;

        /**
         * Creates a RspMachineCode message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspMachineCode
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspMachineCode;

        /**
         * Creates a plain object from a RspMachineCode message. Also converts values to other types if specified.
         * @param message RspMachineCode
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspMachineCode, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspMachineCode to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspMachineCode
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdUploadLicense. */
    interface ICmdUploadLicense {

        /** CmdUploadLicense sessionToken */
        sessionToken?: (string|null);

        /** CmdUploadLicense licenseData */
        licenseData?: (Uint8Array|null);
    }

    /** Represents a CmdUploadLicense. */
    class CmdUploadLicense implements ICmdUploadLicense {

        /**
         * Constructs a new CmdUploadLicense.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdUploadLicense);

        /** CmdUploadLicense sessionToken. */
        public sessionToken: string;

        /** CmdUploadLicense licenseData. */
        public licenseData: Uint8Array;

        /**
         * Encodes the specified CmdUploadLicense message. Does not implicitly {@link usb_control.CmdUploadLicense.verify|verify} messages.
         * @param message CmdUploadLicense message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdUploadLicense, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdUploadLicense message, length delimited. Does not implicitly {@link usb_control.CmdUploadLicense.verify|verify} messages.
         * @param message CmdUploadLicense message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdUploadLicense, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdUploadLicense message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdUploadLicense
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdUploadLicense;

        /**
         * Decodes a CmdUploadLicense message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdUploadLicense
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdUploadLicense;

        /**
         * Creates a CmdUploadLicense message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdUploadLicense
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdUploadLicense;

        /**
         * Creates a plain object from a CmdUploadLicense message. Also converts values to other types if specified.
         * @param message CmdUploadLicense
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdUploadLicense, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdUploadLicense to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdUploadLicense
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspUploadLicense. */
    interface IRspUploadLicense {

        /** RspUploadLicense success */
        success?: (boolean|null);

        /** RspUploadLicense expireTime */
        expireTime?: (number|Long|null);

        /** RspUploadLicense resultCode */
        resultCode?: (number|null);

        /** RspUploadLicense errorMessage */
        errorMessage?: (string|null);
    }

    /** Represents a RspUploadLicense. */
    class RspUploadLicense implements IRspUploadLicense {

        /**
         * Constructs a new RspUploadLicense.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspUploadLicense);

        /** RspUploadLicense success. */
        public success: boolean;

        /** RspUploadLicense expireTime. */
        public expireTime: (number|Long);

        /** RspUploadLicense resultCode. */
        public resultCode: number;

        /** RspUploadLicense errorMessage. */
        public errorMessage: string;

        /**
         * Encodes the specified RspUploadLicense message. Does not implicitly {@link usb_control.RspUploadLicense.verify|verify} messages.
         * @param message RspUploadLicense message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspUploadLicense, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspUploadLicense message, length delimited. Does not implicitly {@link usb_control.RspUploadLicense.verify|verify} messages.
         * @param message RspUploadLicense message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspUploadLicense, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspUploadLicense message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspUploadLicense
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspUploadLicense;

        /**
         * Decodes a RspUploadLicense message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspUploadLicense
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspUploadLicense;

        /**
         * Creates a RspUploadLicense message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspUploadLicense
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspUploadLicense;

        /**
         * Creates a plain object from a RspUploadLicense message. Also converts values to other types if specified.
         * @param message RspUploadLicense
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspUploadLicense, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspUploadLicense to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspUploadLicense
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdLogout. */
    interface ICmdLogout {

        /** CmdLogout sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdLogout. */
    class CmdLogout implements ICmdLogout {

        /**
         * Constructs a new CmdLogout.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdLogout);

        /** CmdLogout sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdLogout message. Does not implicitly {@link usb_control.CmdLogout.verify|verify} messages.
         * @param message CmdLogout message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdLogout, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdLogout message, length delimited. Does not implicitly {@link usb_control.CmdLogout.verify|verify} messages.
         * @param message CmdLogout message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdLogout, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdLogout message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdLogout
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdLogout;

        /**
         * Decodes a CmdLogout message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdLogout
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdLogout;

        /**
         * Creates a CmdLogout message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdLogout
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdLogout;

        /**
         * Creates a plain object from a CmdLogout message. Also converts values to other types if specified.
         * @param message CmdLogout
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdLogout, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdLogout to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdLogout
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a WhitelistDevice. */
    interface IWhitelistDevice {

        /** WhitelistDevice serialNumber */
        serialNumber?: (string|null);

        /** WhitelistDevice vid */
        vid?: (string|null);

        /** WhitelistDevice pid */
        pid?: (string|null);

        /** WhitelistDevice deviceName */
        deviceName?: (string|null);

        /** WhitelistDevice capacityBytes */
        capacityBytes?: (number|Long|null);

        /** WhitelistDevice permission */
        permission?: (string|null);

        /** WhitelistDevice description */
        description?: (string|null);

        /** WhitelistDevice addMethod */
        addMethod?: (string|null);

        /** WhitelistDevice createdAt */
        createdAt?: (number|Long|null);

        /** WhitelistDevice deviceType */
        deviceType?: (string|null);
    }

    /** Represents a WhitelistDevice. */
    class WhitelistDevice implements IWhitelistDevice {

        /**
         * Constructs a new WhitelistDevice.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IWhitelistDevice);

        /** WhitelistDevice serialNumber. */
        public serialNumber: string;

        /** WhitelistDevice vid. */
        public vid: string;

        /** WhitelistDevice pid. */
        public pid: string;

        /** WhitelistDevice deviceName. */
        public deviceName: string;

        /** WhitelistDevice capacityBytes. */
        public capacityBytes: (number|Long);

        /** WhitelistDevice permission. */
        public permission: string;

        /** WhitelistDevice description. */
        public description: string;

        /** WhitelistDevice addMethod. */
        public addMethod: string;

        /** WhitelistDevice createdAt. */
        public createdAt: (number|Long);

        /** WhitelistDevice deviceType. */
        public deviceType: string;

        /**
         * Encodes the specified WhitelistDevice message. Does not implicitly {@link usb_control.WhitelistDevice.verify|verify} messages.
         * @param message WhitelistDevice message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IWhitelistDevice, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified WhitelistDevice message, length delimited. Does not implicitly {@link usb_control.WhitelistDevice.verify|verify} messages.
         * @param message WhitelistDevice message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IWhitelistDevice, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a WhitelistDevice message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns WhitelistDevice
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.WhitelistDevice;

        /**
         * Decodes a WhitelistDevice message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns WhitelistDevice
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.WhitelistDevice;

        /**
         * Creates a WhitelistDevice message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns WhitelistDevice
         */
        public static fromObject(object: { [k: string]: any }): usb_control.WhitelistDevice;

        /**
         * Creates a plain object from a WhitelistDevice message. Also converts values to other types if specified.
         * @param message WhitelistDevice
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.WhitelistDevice, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this WhitelistDevice to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for WhitelistDevice
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdListWhitelist. */
    interface ICmdListWhitelist {

        /** CmdListWhitelist sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdListWhitelist. */
    class CmdListWhitelist implements ICmdListWhitelist {

        /**
         * Constructs a new CmdListWhitelist.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdListWhitelist);

        /** CmdListWhitelist sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdListWhitelist message. Does not implicitly {@link usb_control.CmdListWhitelist.verify|verify} messages.
         * @param message CmdListWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdListWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdListWhitelist message, length delimited. Does not implicitly {@link usb_control.CmdListWhitelist.verify|verify} messages.
         * @param message CmdListWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdListWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdListWhitelist message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdListWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdListWhitelist;

        /**
         * Decodes a CmdListWhitelist message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdListWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdListWhitelist;

        /**
         * Creates a CmdListWhitelist message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdListWhitelist
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdListWhitelist;

        /**
         * Creates a plain object from a CmdListWhitelist message. Also converts values to other types if specified.
         * @param message CmdListWhitelist
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdListWhitelist, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdListWhitelist to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdListWhitelist
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspListWhitelist. */
    interface IRspListWhitelist {

        /** RspListWhitelist devices */
        devices?: (usb_control.IWhitelistDevice[]|null);
    }

    /** Represents a RspListWhitelist. */
    class RspListWhitelist implements IRspListWhitelist {

        /**
         * Constructs a new RspListWhitelist.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspListWhitelist);

        /** RspListWhitelist devices. */
        public devices: usb_control.IWhitelistDevice[];

        /**
         * Encodes the specified RspListWhitelist message. Does not implicitly {@link usb_control.RspListWhitelist.verify|verify} messages.
         * @param message RspListWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspListWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspListWhitelist message, length delimited. Does not implicitly {@link usb_control.RspListWhitelist.verify|verify} messages.
         * @param message RspListWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspListWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspListWhitelist message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspListWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspListWhitelist;

        /**
         * Decodes a RspListWhitelist message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspListWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspListWhitelist;

        /**
         * Creates a RspListWhitelist message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspListWhitelist
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspListWhitelist;

        /**
         * Creates a plain object from a RspListWhitelist message. Also converts values to other types if specified.
         * @param message RspListWhitelist
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspListWhitelist, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspListWhitelist to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspListWhitelist
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ConnectedDevice. */
    interface IConnectedDevice {

        /** ConnectedDevice serialNumber */
        serialNumber?: (string|null);

        /** ConnectedDevice deviceName */
        deviceName?: (string|null);

        /** ConnectedDevice vid */
        vid?: (string|null);

        /** ConnectedDevice pid */
        pid?: (string|null);

        /** ConnectedDevice capacityBytes */
        capacityBytes?: (number|Long|null);

        /** ConnectedDevice deviceType */
        deviceType?: (string|null);

        /** ConnectedDevice interfaceType */
        interfaceType?: (string|null);

        /** ConnectedDevice admissionStatus */
        admissionStatus?: (string|null);

        /** ConnectedDevice failReason */
        failReason?: (string|null);
    }

    /** Represents a ConnectedDevice. */
    class ConnectedDevice implements IConnectedDevice {

        /**
         * Constructs a new ConnectedDevice.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IConnectedDevice);

        /** ConnectedDevice serialNumber. */
        public serialNumber: string;

        /** ConnectedDevice deviceName. */
        public deviceName: string;

        /** ConnectedDevice vid. */
        public vid: string;

        /** ConnectedDevice pid. */
        public pid: string;

        /** ConnectedDevice capacityBytes. */
        public capacityBytes: (number|Long);

        /** ConnectedDevice deviceType. */
        public deviceType: string;

        /** ConnectedDevice interfaceType. */
        public interfaceType: string;

        /** ConnectedDevice admissionStatus. */
        public admissionStatus: string;

        /** ConnectedDevice failReason. */
        public failReason: string;

        /**
         * Encodes the specified ConnectedDevice message. Does not implicitly {@link usb_control.ConnectedDevice.verify|verify} messages.
         * @param message ConnectedDevice message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IConnectedDevice, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ConnectedDevice message, length delimited. Does not implicitly {@link usb_control.ConnectedDevice.verify|verify} messages.
         * @param message ConnectedDevice message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IConnectedDevice, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ConnectedDevice message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ConnectedDevice
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.ConnectedDevice;

        /**
         * Decodes a ConnectedDevice message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ConnectedDevice
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.ConnectedDevice;

        /**
         * Creates a ConnectedDevice message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ConnectedDevice
         */
        public static fromObject(object: { [k: string]: any }): usb_control.ConnectedDevice;

        /**
         * Creates a plain object from a ConnectedDevice message. Also converts values to other types if specified.
         * @param message ConnectedDevice
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.ConnectedDevice, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ConnectedDevice to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ConnectedDevice
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdGetConnectedDevices. */
    interface ICmdGetConnectedDevices {

        /** CmdGetConnectedDevices sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdGetConnectedDevices. */
    class CmdGetConnectedDevices implements ICmdGetConnectedDevices {

        /**
         * Constructs a new CmdGetConnectedDevices.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdGetConnectedDevices);

        /** CmdGetConnectedDevices sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdGetConnectedDevices message. Does not implicitly {@link usb_control.CmdGetConnectedDevices.verify|verify} messages.
         * @param message CmdGetConnectedDevices message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdGetConnectedDevices, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdGetConnectedDevices message, length delimited. Does not implicitly {@link usb_control.CmdGetConnectedDevices.verify|verify} messages.
         * @param message CmdGetConnectedDevices message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdGetConnectedDevices, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdGetConnectedDevices message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdGetConnectedDevices
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdGetConnectedDevices;

        /**
         * Decodes a CmdGetConnectedDevices message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdGetConnectedDevices
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdGetConnectedDevices;

        /**
         * Creates a CmdGetConnectedDevices message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdGetConnectedDevices
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdGetConnectedDevices;

        /**
         * Creates a plain object from a CmdGetConnectedDevices message. Also converts values to other types if specified.
         * @param message CmdGetConnectedDevices
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdGetConnectedDevices, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdGetConnectedDevices to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdGetConnectedDevices
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspConnectedDevices. */
    interface IRspConnectedDevices {

        /** RspConnectedDevices devices */
        devices?: (usb_control.IConnectedDevice[]|null);
    }

    /** Represents a RspConnectedDevices. */
    class RspConnectedDevices implements IRspConnectedDevices {

        /**
         * Constructs a new RspConnectedDevices.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspConnectedDevices);

        /** RspConnectedDevices devices. */
        public devices: usb_control.IConnectedDevice[];

        /**
         * Encodes the specified RspConnectedDevices message. Does not implicitly {@link usb_control.RspConnectedDevices.verify|verify} messages.
         * @param message RspConnectedDevices message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspConnectedDevices, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspConnectedDevices message, length delimited. Does not implicitly {@link usb_control.RspConnectedDevices.verify|verify} messages.
         * @param message RspConnectedDevices message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspConnectedDevices, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspConnectedDevices message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspConnectedDevices
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspConnectedDevices;

        /**
         * Decodes a RspConnectedDevices message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspConnectedDevices
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspConnectedDevices;

        /**
         * Creates a RspConnectedDevices message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspConnectedDevices
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspConnectedDevices;

        /**
         * Creates a plain object from a RspConnectedDevices message. Also converts values to other types if specified.
         * @param message RspConnectedDevices
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspConnectedDevices, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspConnectedDevices to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspConnectedDevices
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdAddWhitelist. */
    interface ICmdAddWhitelist {

        /** CmdAddWhitelist sessionToken */
        sessionToken?: (string|null);

        /** CmdAddWhitelist serialNumber */
        serialNumber?: (string|null);

        /** CmdAddWhitelist vid */
        vid?: (string|null);

        /** CmdAddWhitelist pid */
        pid?: (string|null);

        /** CmdAddWhitelist deviceName */
        deviceName?: (string|null);

        /** CmdAddWhitelist capacityBytes */
        capacityBytes?: (number|Long|null);

        /** CmdAddWhitelist permission */
        permission?: (string|null);

        /** CmdAddWhitelist description */
        description?: (string|null);

        /** CmdAddWhitelist addMethod */
        addMethod?: (string|null);

        /** CmdAddWhitelist deviceType */
        deviceType?: (string|null);
    }

    /** Represents a CmdAddWhitelist. */
    class CmdAddWhitelist implements ICmdAddWhitelist {

        /**
         * Constructs a new CmdAddWhitelist.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdAddWhitelist);

        /** CmdAddWhitelist sessionToken. */
        public sessionToken: string;

        /** CmdAddWhitelist serialNumber. */
        public serialNumber: string;

        /** CmdAddWhitelist vid. */
        public vid: string;

        /** CmdAddWhitelist pid. */
        public pid: string;

        /** CmdAddWhitelist deviceName. */
        public deviceName: string;

        /** CmdAddWhitelist capacityBytes. */
        public capacityBytes: (number|Long);

        /** CmdAddWhitelist permission. */
        public permission: string;

        /** CmdAddWhitelist description. */
        public description: string;

        /** CmdAddWhitelist addMethod. */
        public addMethod: string;

        /** CmdAddWhitelist deviceType. */
        public deviceType: string;

        /**
         * Encodes the specified CmdAddWhitelist message. Does not implicitly {@link usb_control.CmdAddWhitelist.verify|verify} messages.
         * @param message CmdAddWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdAddWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdAddWhitelist message, length delimited. Does not implicitly {@link usb_control.CmdAddWhitelist.verify|verify} messages.
         * @param message CmdAddWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdAddWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdAddWhitelist message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdAddWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdAddWhitelist;

        /**
         * Decodes a CmdAddWhitelist message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdAddWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdAddWhitelist;

        /**
         * Creates a CmdAddWhitelist message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdAddWhitelist
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdAddWhitelist;

        /**
         * Creates a plain object from a CmdAddWhitelist message. Also converts values to other types if specified.
         * @param message CmdAddWhitelist
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdAddWhitelist, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdAddWhitelist to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdAddWhitelist
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdRemoveWhitelist. */
    interface ICmdRemoveWhitelist {

        /** CmdRemoveWhitelist sessionToken */
        sessionToken?: (string|null);

        /** CmdRemoveWhitelist serialNumber */
        serialNumber?: (string|null);
    }

    /** Represents a CmdRemoveWhitelist. */
    class CmdRemoveWhitelist implements ICmdRemoveWhitelist {

        /**
         * Constructs a new CmdRemoveWhitelist.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdRemoveWhitelist);

        /** CmdRemoveWhitelist sessionToken. */
        public sessionToken: string;

        /** CmdRemoveWhitelist serialNumber. */
        public serialNumber: string;

        /**
         * Encodes the specified CmdRemoveWhitelist message. Does not implicitly {@link usb_control.CmdRemoveWhitelist.verify|verify} messages.
         * @param message CmdRemoveWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdRemoveWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdRemoveWhitelist message, length delimited. Does not implicitly {@link usb_control.CmdRemoveWhitelist.verify|verify} messages.
         * @param message CmdRemoveWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdRemoveWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdRemoveWhitelist message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdRemoveWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdRemoveWhitelist;

        /**
         * Decodes a CmdRemoveWhitelist message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdRemoveWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdRemoveWhitelist;

        /**
         * Creates a CmdRemoveWhitelist message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdRemoveWhitelist
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdRemoveWhitelist;

        /**
         * Creates a plain object from a CmdRemoveWhitelist message. Also converts values to other types if specified.
         * @param message CmdRemoveWhitelist
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdRemoveWhitelist, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdRemoveWhitelist to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdRemoveWhitelist
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdUpdateWhitelist. */
    interface ICmdUpdateWhitelist {

        /** CmdUpdateWhitelist sessionToken */
        sessionToken?: (string|null);

        /** CmdUpdateWhitelist serialNumber */
        serialNumber?: (string|null);

        /** CmdUpdateWhitelist permission */
        permission?: (string|null);

        /** CmdUpdateWhitelist description */
        description?: (string|null);
    }

    /** Represents a CmdUpdateWhitelist. */
    class CmdUpdateWhitelist implements ICmdUpdateWhitelist {

        /**
         * Constructs a new CmdUpdateWhitelist.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdUpdateWhitelist);

        /** CmdUpdateWhitelist sessionToken. */
        public sessionToken: string;

        /** CmdUpdateWhitelist serialNumber. */
        public serialNumber: string;

        /** CmdUpdateWhitelist permission. */
        public permission: string;

        /** CmdUpdateWhitelist description. */
        public description: string;

        /**
         * Encodes the specified CmdUpdateWhitelist message. Does not implicitly {@link usb_control.CmdUpdateWhitelist.verify|verify} messages.
         * @param message CmdUpdateWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdUpdateWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdUpdateWhitelist message, length delimited. Does not implicitly {@link usb_control.CmdUpdateWhitelist.verify|verify} messages.
         * @param message CmdUpdateWhitelist message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdUpdateWhitelist, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdUpdateWhitelist message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdUpdateWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdUpdateWhitelist;

        /**
         * Decodes a CmdUpdateWhitelist message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdUpdateWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdUpdateWhitelist;

        /**
         * Creates a CmdUpdateWhitelist message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdUpdateWhitelist
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdUpdateWhitelist;

        /**
         * Creates a plain object from a CmdUpdateWhitelist message. Also converts values to other types if specified.
         * @param message CmdUpdateWhitelist
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdUpdateWhitelist, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdUpdateWhitelist to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdUpdateWhitelist
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a FileTypeBlacklistItem. */
    interface IFileTypeBlacklistItem {

        /** FileTypeBlacklistItem extension */
        extension?: (string|null);

        /** FileTypeBlacklistItem description */
        description?: (string|null);

        /** FileTypeBlacklistItem isDefault */
        isDefault?: (boolean|null);

        /** FileTypeBlacklistItem createdAt */
        createdAt?: (number|Long|null);
    }

    /** Represents a FileTypeBlacklistItem. */
    class FileTypeBlacklistItem implements IFileTypeBlacklistItem {

        /**
         * Constructs a new FileTypeBlacklistItem.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IFileTypeBlacklistItem);

        /** FileTypeBlacklistItem extension. */
        public extension: string;

        /** FileTypeBlacklistItem description. */
        public description: string;

        /** FileTypeBlacklistItem isDefault. */
        public isDefault: boolean;

        /** FileTypeBlacklistItem createdAt. */
        public createdAt: (number|Long);

        /**
         * Encodes the specified FileTypeBlacklistItem message. Does not implicitly {@link usb_control.FileTypeBlacklistItem.verify|verify} messages.
         * @param message FileTypeBlacklistItem message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IFileTypeBlacklistItem, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified FileTypeBlacklistItem message, length delimited. Does not implicitly {@link usb_control.FileTypeBlacklistItem.verify|verify} messages.
         * @param message FileTypeBlacklistItem message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IFileTypeBlacklistItem, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a FileTypeBlacklistItem message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns FileTypeBlacklistItem
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.FileTypeBlacklistItem;

        /**
         * Decodes a FileTypeBlacklistItem message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns FileTypeBlacklistItem
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.FileTypeBlacklistItem;

        /**
         * Creates a FileTypeBlacklistItem message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns FileTypeBlacklistItem
         */
        public static fromObject(object: { [k: string]: any }): usb_control.FileTypeBlacklistItem;

        /**
         * Creates a plain object from a FileTypeBlacklistItem message. Also converts values to other types if specified.
         * @param message FileTypeBlacklistItem
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.FileTypeBlacklistItem, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this FileTypeBlacklistItem to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for FileTypeBlacklistItem
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdGetFilePolicy. */
    interface ICmdGetFilePolicy {

        /** CmdGetFilePolicy sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdGetFilePolicy. */
    class CmdGetFilePolicy implements ICmdGetFilePolicy {

        /**
         * Constructs a new CmdGetFilePolicy.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdGetFilePolicy);

        /** CmdGetFilePolicy sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdGetFilePolicy message. Does not implicitly {@link usb_control.CmdGetFilePolicy.verify|verify} messages.
         * @param message CmdGetFilePolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdGetFilePolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdGetFilePolicy message, length delimited. Does not implicitly {@link usb_control.CmdGetFilePolicy.verify|verify} messages.
         * @param message CmdGetFilePolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdGetFilePolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdGetFilePolicy message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdGetFilePolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdGetFilePolicy;

        /**
         * Decodes a CmdGetFilePolicy message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdGetFilePolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdGetFilePolicy;

        /**
         * Creates a CmdGetFilePolicy message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdGetFilePolicy
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdGetFilePolicy;

        /**
         * Creates a plain object from a CmdGetFilePolicy message. Also converts values to other types if specified.
         * @param message CmdGetFilePolicy
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdGetFilePolicy, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdGetFilePolicy to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdGetFilePolicy
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspFilePolicy. */
    interface IRspFilePolicy {

        /** RspFilePolicy execControlEnabled */
        execControlEnabled?: (boolean|null);

        /** RspFilePolicy autoReadControlEnabled */
        autoReadControlEnabled?: (boolean|null);

        /** RspFilePolicy fileTypeBlacklistEnabled */
        fileTypeBlacklistEnabled?: (boolean|null);

        /** RspFilePolicy blacklist */
        blacklist?: (usb_control.IFileTypeBlacklistItem[]|null);
    }

    /** Represents a RspFilePolicy. */
    class RspFilePolicy implements IRspFilePolicy {

        /**
         * Constructs a new RspFilePolicy.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspFilePolicy);

        /** RspFilePolicy execControlEnabled. */
        public execControlEnabled: boolean;

        /** RspFilePolicy autoReadControlEnabled. */
        public autoReadControlEnabled: boolean;

        /** RspFilePolicy fileTypeBlacklistEnabled. */
        public fileTypeBlacklistEnabled: boolean;

        /** RspFilePolicy blacklist. */
        public blacklist: usb_control.IFileTypeBlacklistItem[];

        /**
         * Encodes the specified RspFilePolicy message. Does not implicitly {@link usb_control.RspFilePolicy.verify|verify} messages.
         * @param message RspFilePolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspFilePolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspFilePolicy message, length delimited. Does not implicitly {@link usb_control.RspFilePolicy.verify|verify} messages.
         * @param message RspFilePolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspFilePolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspFilePolicy message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspFilePolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspFilePolicy;

        /**
         * Decodes a RspFilePolicy message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspFilePolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspFilePolicy;

        /**
         * Creates a RspFilePolicy message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspFilePolicy
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspFilePolicy;

        /**
         * Creates a plain object from a RspFilePolicy message. Also converts values to other types if specified.
         * @param message RspFilePolicy
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspFilePolicy, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspFilePolicy to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspFilePolicy
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdUpdateFilePolicySwitch. */
    interface ICmdUpdateFilePolicySwitch {

        /** CmdUpdateFilePolicySwitch sessionToken */
        sessionToken?: (string|null);

        /** CmdUpdateFilePolicySwitch policyKey */
        policyKey?: (string|null);

        /** CmdUpdateFilePolicySwitch enabled */
        enabled?: (boolean|null);
    }

    /** Represents a CmdUpdateFilePolicySwitch. */
    class CmdUpdateFilePolicySwitch implements ICmdUpdateFilePolicySwitch {

        /**
         * Constructs a new CmdUpdateFilePolicySwitch.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdUpdateFilePolicySwitch);

        /** CmdUpdateFilePolicySwitch sessionToken. */
        public sessionToken: string;

        /** CmdUpdateFilePolicySwitch policyKey. */
        public policyKey: string;

        /** CmdUpdateFilePolicySwitch enabled. */
        public enabled: boolean;

        /**
         * Encodes the specified CmdUpdateFilePolicySwitch message. Does not implicitly {@link usb_control.CmdUpdateFilePolicySwitch.verify|verify} messages.
         * @param message CmdUpdateFilePolicySwitch message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdUpdateFilePolicySwitch, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdUpdateFilePolicySwitch message, length delimited. Does not implicitly {@link usb_control.CmdUpdateFilePolicySwitch.verify|verify} messages.
         * @param message CmdUpdateFilePolicySwitch message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdUpdateFilePolicySwitch, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdUpdateFilePolicySwitch message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdUpdateFilePolicySwitch
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdUpdateFilePolicySwitch;

        /**
         * Decodes a CmdUpdateFilePolicySwitch message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdUpdateFilePolicySwitch
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdUpdateFilePolicySwitch;

        /**
         * Creates a CmdUpdateFilePolicySwitch message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdUpdateFilePolicySwitch
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdUpdateFilePolicySwitch;

        /**
         * Creates a plain object from a CmdUpdateFilePolicySwitch message. Also converts values to other types if specified.
         * @param message CmdUpdateFilePolicySwitch
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdUpdateFilePolicySwitch, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdUpdateFilePolicySwitch to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdUpdateFilePolicySwitch
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdAddBlacklistExtension. */
    interface ICmdAddBlacklistExtension {

        /** CmdAddBlacklistExtension sessionToken */
        sessionToken?: (string|null);

        /** CmdAddBlacklistExtension extension */
        extension?: (string|null);

        /** CmdAddBlacklistExtension description */
        description?: (string|null);
    }

    /** Represents a CmdAddBlacklistExtension. */
    class CmdAddBlacklistExtension implements ICmdAddBlacklistExtension {

        /**
         * Constructs a new CmdAddBlacklistExtension.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdAddBlacklistExtension);

        /** CmdAddBlacklistExtension sessionToken. */
        public sessionToken: string;

        /** CmdAddBlacklistExtension extension. */
        public extension: string;

        /** CmdAddBlacklistExtension description. */
        public description: string;

        /**
         * Encodes the specified CmdAddBlacklistExtension message. Does not implicitly {@link usb_control.CmdAddBlacklistExtension.verify|verify} messages.
         * @param message CmdAddBlacklistExtension message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdAddBlacklistExtension, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdAddBlacklistExtension message, length delimited. Does not implicitly {@link usb_control.CmdAddBlacklistExtension.verify|verify} messages.
         * @param message CmdAddBlacklistExtension message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdAddBlacklistExtension, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdAddBlacklistExtension message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdAddBlacklistExtension
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdAddBlacklistExtension;

        /**
         * Decodes a CmdAddBlacklistExtension message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdAddBlacklistExtension
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdAddBlacklistExtension;

        /**
         * Creates a CmdAddBlacklistExtension message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdAddBlacklistExtension
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdAddBlacklistExtension;

        /**
         * Creates a plain object from a CmdAddBlacklistExtension message. Also converts values to other types if specified.
         * @param message CmdAddBlacklistExtension
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdAddBlacklistExtension, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdAddBlacklistExtension to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdAddBlacklistExtension
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdRemoveBlacklistExtension. */
    interface ICmdRemoveBlacklistExtension {

        /** CmdRemoveBlacklistExtension sessionToken */
        sessionToken?: (string|null);

        /** CmdRemoveBlacklistExtension extension */
        extension?: (string|null);
    }

    /** Represents a CmdRemoveBlacklistExtension. */
    class CmdRemoveBlacklistExtension implements ICmdRemoveBlacklistExtension {

        /**
         * Constructs a new CmdRemoveBlacklistExtension.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdRemoveBlacklistExtension);

        /** CmdRemoveBlacklistExtension sessionToken. */
        public sessionToken: string;

        /** CmdRemoveBlacklistExtension extension. */
        public extension: string;

        /**
         * Encodes the specified CmdRemoveBlacklistExtension message. Does not implicitly {@link usb_control.CmdRemoveBlacklistExtension.verify|verify} messages.
         * @param message CmdRemoveBlacklistExtension message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdRemoveBlacklistExtension, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdRemoveBlacklistExtension message, length delimited. Does not implicitly {@link usb_control.CmdRemoveBlacklistExtension.verify|verify} messages.
         * @param message CmdRemoveBlacklistExtension message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdRemoveBlacklistExtension, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdRemoveBlacklistExtension message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdRemoveBlacklistExtension
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdRemoveBlacklistExtension;

        /**
         * Decodes a CmdRemoveBlacklistExtension message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdRemoveBlacklistExtension
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdRemoveBlacklistExtension;

        /**
         * Creates a CmdRemoveBlacklistExtension message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdRemoveBlacklistExtension
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdRemoveBlacklistExtension;

        /**
         * Creates a plain object from a CmdRemoveBlacklistExtension message. Also converts values to other types if specified.
         * @param message CmdRemoveBlacklistExtension
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdRemoveBlacklistExtension, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdRemoveBlacklistExtension to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdRemoveBlacklistExtension
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdExportPolicy. */
    interface ICmdExportPolicy {

        /** CmdExportPolicy sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdExportPolicy. */
    class CmdExportPolicy implements ICmdExportPolicy {

        /**
         * Constructs a new CmdExportPolicy.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdExportPolicy);

        /** CmdExportPolicy sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdExportPolicy message. Does not implicitly {@link usb_control.CmdExportPolicy.verify|verify} messages.
         * @param message CmdExportPolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdExportPolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdExportPolicy message, length delimited. Does not implicitly {@link usb_control.CmdExportPolicy.verify|verify} messages.
         * @param message CmdExportPolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdExportPolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdExportPolicy message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdExportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdExportPolicy;

        /**
         * Decodes a CmdExportPolicy message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdExportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdExportPolicy;

        /**
         * Creates a CmdExportPolicy message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdExportPolicy
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdExportPolicy;

        /**
         * Creates a plain object from a CmdExportPolicy message. Also converts values to other types if specified.
         * @param message CmdExportPolicy
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdExportPolicy, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdExportPolicy to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdExportPolicy
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspExportPolicy. */
    interface IRspExportPolicy {

        /** RspExportPolicy success */
        success?: (boolean|null);

        /** RspExportPolicy policyData */
        policyData?: (Uint8Array|null);

        /** RspExportPolicy resultCode */
        resultCode?: (number|null);

        /** RspExportPolicy errorMessage */
        errorMessage?: (string|null);
    }

    /** Represents a RspExportPolicy. */
    class RspExportPolicy implements IRspExportPolicy {

        /**
         * Constructs a new RspExportPolicy.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspExportPolicy);

        /** RspExportPolicy success. */
        public success: boolean;

        /** RspExportPolicy policyData. */
        public policyData: Uint8Array;

        /** RspExportPolicy resultCode. */
        public resultCode: number;

        /** RspExportPolicy errorMessage. */
        public errorMessage: string;

        /**
         * Encodes the specified RspExportPolicy message. Does not implicitly {@link usb_control.RspExportPolicy.verify|verify} messages.
         * @param message RspExportPolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspExportPolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspExportPolicy message, length delimited. Does not implicitly {@link usb_control.RspExportPolicy.verify|verify} messages.
         * @param message RspExportPolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspExportPolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspExportPolicy message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspExportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspExportPolicy;

        /**
         * Decodes a RspExportPolicy message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspExportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspExportPolicy;

        /**
         * Creates a RspExportPolicy message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspExportPolicy
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspExportPolicy;

        /**
         * Creates a plain object from a RspExportPolicy message. Also converts values to other types if specified.
         * @param message RspExportPolicy
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspExportPolicy, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspExportPolicy to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspExportPolicy
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdImportPolicy. */
    interface ICmdImportPolicy {

        /** CmdImportPolicy sessionToken */
        sessionToken?: (string|null);

        /** CmdImportPolicy policyData */
        policyData?: (Uint8Array|null);
    }

    /** Represents a CmdImportPolicy. */
    class CmdImportPolicy implements ICmdImportPolicy {

        /**
         * Constructs a new CmdImportPolicy.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdImportPolicy);

        /** CmdImportPolicy sessionToken. */
        public sessionToken: string;

        /** CmdImportPolicy policyData. */
        public policyData: Uint8Array;

        /**
         * Encodes the specified CmdImportPolicy message. Does not implicitly {@link usb_control.CmdImportPolicy.verify|verify} messages.
         * @param message CmdImportPolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdImportPolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdImportPolicy message, length delimited. Does not implicitly {@link usb_control.CmdImportPolicy.verify|verify} messages.
         * @param message CmdImportPolicy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdImportPolicy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdImportPolicy message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdImportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdImportPolicy;

        /**
         * Decodes a CmdImportPolicy message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdImportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdImportPolicy;

        /**
         * Creates a CmdImportPolicy message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdImportPolicy
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdImportPolicy;

        /**
         * Creates a plain object from a CmdImportPolicy message. Also converts values to other types if specified.
         * @param message CmdImportPolicy
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdImportPolicy, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdImportPolicy to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdImportPolicy
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdQueryLogs. */
    interface ICmdQueryLogs {

        /** CmdQueryLogs sessionToken */
        sessionToken?: (string|null);

        /** CmdQueryLogs logType */
        logType?: (string|null);

        /** CmdQueryLogs startTime */
        startTime?: (number|Long|null);

        /** CmdQueryLogs endTime */
        endTime?: (number|Long|null);

        /** CmdQueryLogs keyword */
        keyword?: (string|null);

        /** CmdQueryLogs eventType */
        eventType?: (string|null);

        /** CmdQueryLogs page */
        page?: (number|null);

        /** CmdQueryLogs pageSize */
        pageSize?: (number|null);
    }

    /** Represents a CmdQueryLogs. */
    class CmdQueryLogs implements ICmdQueryLogs {

        /**
         * Constructs a new CmdQueryLogs.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdQueryLogs);

        /** CmdQueryLogs sessionToken. */
        public sessionToken: string;

        /** CmdQueryLogs logType. */
        public logType: string;

        /** CmdQueryLogs startTime. */
        public startTime: (number|Long);

        /** CmdQueryLogs endTime. */
        public endTime: (number|Long);

        /** CmdQueryLogs keyword. */
        public keyword: string;

        /** CmdQueryLogs eventType. */
        public eventType: string;

        /** CmdQueryLogs page. */
        public page: number;

        /** CmdQueryLogs pageSize. */
        public pageSize: number;

        /**
         * Encodes the specified CmdQueryLogs message. Does not implicitly {@link usb_control.CmdQueryLogs.verify|verify} messages.
         * @param message CmdQueryLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdQueryLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdQueryLogs message, length delimited. Does not implicitly {@link usb_control.CmdQueryLogs.verify|verify} messages.
         * @param message CmdQueryLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdQueryLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdQueryLogs message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdQueryLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdQueryLogs;

        /**
         * Decodes a CmdQueryLogs message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdQueryLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdQueryLogs;

        /**
         * Creates a CmdQueryLogs message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdQueryLogs
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdQueryLogs;

        /**
         * Creates a plain object from a CmdQueryLogs message. Also converts values to other types if specified.
         * @param message CmdQueryLogs
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdQueryLogs, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdQueryLogs to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdQueryLogs
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a UsbAuditLogEntry. */
    interface IUsbAuditLogEntry {

        /** UsbAuditLogEntry id */
        id?: (number|Long|null);

        /** UsbAuditLogEntry eventTime */
        eventTime?: (number|Long|null);

        /** UsbAuditLogEntry deviceSn */
        deviceSn?: (string|null);

        /** UsbAuditLogEntry deviceName */
        deviceName?: (string|null);

        /** UsbAuditLogEntry deviceType */
        deviceType?: (string|null);

        /** UsbAuditLogEntry interfaceType */
        interfaceType?: (string|null);

        /** UsbAuditLogEntry eventType */
        eventType?: (string|null);

        /** UsbAuditLogEntry permission */
        permission?: (string|null);

        /** UsbAuditLogEntry capacityBytes */
        capacityBytes?: (number|Long|null);

        /** UsbAuditLogEntry filePath */
        filePath?: (string|null);

        /** UsbAuditLogEntry matchedPolicy */
        matchedPolicy?: (string|null);

        /** UsbAuditLogEntry result */
        result?: (string|null);

        /** UsbAuditLogEntry failReason */
        failReason?: (string|null);

        /** UsbAuditLogEntry detail */
        detail?: (string|null);
    }

    /** Represents a UsbAuditLogEntry. */
    class UsbAuditLogEntry implements IUsbAuditLogEntry {

        /**
         * Constructs a new UsbAuditLogEntry.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IUsbAuditLogEntry);

        /** UsbAuditLogEntry id. */
        public id: (number|Long);

        /** UsbAuditLogEntry eventTime. */
        public eventTime: (number|Long);

        /** UsbAuditLogEntry deviceSn. */
        public deviceSn: string;

        /** UsbAuditLogEntry deviceName. */
        public deviceName: string;

        /** UsbAuditLogEntry deviceType. */
        public deviceType: string;

        /** UsbAuditLogEntry interfaceType. */
        public interfaceType: string;

        /** UsbAuditLogEntry eventType. */
        public eventType: string;

        /** UsbAuditLogEntry permission. */
        public permission: string;

        /** UsbAuditLogEntry capacityBytes. */
        public capacityBytes: (number|Long);

        /** UsbAuditLogEntry filePath. */
        public filePath: string;

        /** UsbAuditLogEntry matchedPolicy. */
        public matchedPolicy: string;

        /** UsbAuditLogEntry result. */
        public result: string;

        /** UsbAuditLogEntry failReason. */
        public failReason: string;

        /** UsbAuditLogEntry detail. */
        public detail: string;

        /**
         * Encodes the specified UsbAuditLogEntry message. Does not implicitly {@link usb_control.UsbAuditLogEntry.verify|verify} messages.
         * @param message UsbAuditLogEntry message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IUsbAuditLogEntry, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified UsbAuditLogEntry message, length delimited. Does not implicitly {@link usb_control.UsbAuditLogEntry.verify|verify} messages.
         * @param message UsbAuditLogEntry message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IUsbAuditLogEntry, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a UsbAuditLogEntry message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns UsbAuditLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.UsbAuditLogEntry;

        /**
         * Decodes a UsbAuditLogEntry message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns UsbAuditLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.UsbAuditLogEntry;

        /**
         * Creates a UsbAuditLogEntry message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns UsbAuditLogEntry
         */
        public static fromObject(object: { [k: string]: any }): usb_control.UsbAuditLogEntry;

        /**
         * Creates a plain object from a UsbAuditLogEntry message. Also converts values to other types if specified.
         * @param message UsbAuditLogEntry
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.UsbAuditLogEntry, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this UsbAuditLogEntry to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for UsbAuditLogEntry
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a MalwareLogEntry. */
    interface IMalwareLogEntry {

        /** MalwareLogEntry id */
        id?: (number|Long|null);

        /** MalwareLogEntry scanTime */
        scanTime?: (number|Long|null);

        /** MalwareLogEntry deviceSn */
        deviceSn?: (string|null);

        /** MalwareLogEntry deviceName */
        deviceName?: (string|null);

        /** MalwareLogEntry filePath */
        filePath?: (string|null);

        /** MalwareLogEntry scanResult */
        scanResult?: (string|null);

        /** MalwareLogEntry virusName */
        virusName?: (string|null);

        /** MalwareLogEntry virusDbVersion */
        virusDbVersion?: (string|null);

        /** MalwareLogEntry processResult */
        processResult?: (string|null);

        /** MalwareLogEntry failReason */
        failReason?: (string|null);

        /** MalwareLogEntry detail */
        detail?: (string|null);
    }

    /** Represents a MalwareLogEntry. */
    class MalwareLogEntry implements IMalwareLogEntry {

        /**
         * Constructs a new MalwareLogEntry.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IMalwareLogEntry);

        /** MalwareLogEntry id. */
        public id: (number|Long);

        /** MalwareLogEntry scanTime. */
        public scanTime: (number|Long);

        /** MalwareLogEntry deviceSn. */
        public deviceSn: string;

        /** MalwareLogEntry deviceName. */
        public deviceName: string;

        /** MalwareLogEntry filePath. */
        public filePath: string;

        /** MalwareLogEntry scanResult. */
        public scanResult: string;

        /** MalwareLogEntry virusName. */
        public virusName: string;

        /** MalwareLogEntry virusDbVersion. */
        public virusDbVersion: string;

        /** MalwareLogEntry processResult. */
        public processResult: string;

        /** MalwareLogEntry failReason. */
        public failReason: string;

        /** MalwareLogEntry detail. */
        public detail: string;

        /**
         * Encodes the specified MalwareLogEntry message. Does not implicitly {@link usb_control.MalwareLogEntry.verify|verify} messages.
         * @param message MalwareLogEntry message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IMalwareLogEntry, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified MalwareLogEntry message, length delimited. Does not implicitly {@link usb_control.MalwareLogEntry.verify|verify} messages.
         * @param message MalwareLogEntry message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IMalwareLogEntry, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a MalwareLogEntry message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns MalwareLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.MalwareLogEntry;

        /**
         * Decodes a MalwareLogEntry message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns MalwareLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.MalwareLogEntry;

        /**
         * Creates a MalwareLogEntry message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns MalwareLogEntry
         */
        public static fromObject(object: { [k: string]: any }): usb_control.MalwareLogEntry;

        /**
         * Creates a plain object from a MalwareLogEntry message. Also converts values to other types if specified.
         * @param message MalwareLogEntry
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.MalwareLogEntry, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this MalwareLogEntry to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for MalwareLogEntry
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of an OperationLogEntry. */
    interface IOperationLogEntry {

        /** OperationLogEntry id */
        id?: (number|Long|null);

        /** OperationLogEntry opTime */
        opTime?: (number|Long|null);

        /** OperationLogEntry username */
        username?: (string|null);

        /** OperationLogEntry role */
        role?: (string|null);

        /** OperationLogEntry logCategory */
        logCategory?: (string|null);

        /** OperationLogEntry actionType */
        actionType?: (string|null);

        /** OperationLogEntry target */
        target?: (string|null);

        /** OperationLogEntry relatedFile */
        relatedFile?: (string|null);

        /** OperationLogEntry relatedVersion */
        relatedVersion?: (string|null);

        /** OperationLogEntry result */
        result?: (string|null);

        /** OperationLogEntry failReason */
        failReason?: (string|null);

        /** OperationLogEntry detail */
        detail?: (string|null);

        /** OperationLogEntry sourceIp */
        sourceIp?: (string|null);

        /** OperationLogEntry appVersion */
        appVersion?: (string|null);

        /** OperationLogEntry sessionId */
        sessionId?: (string|null);

        /** OperationLogEntry requestId */
        requestId?: (string|null);

        /** OperationLogEntry beforeValue */
        beforeValue?: (string|null);

        /** OperationLogEntry afterValue */
        afterValue?: (string|null);
    }

    /** Represents an OperationLogEntry. */
    class OperationLogEntry implements IOperationLogEntry {

        /**
         * Constructs a new OperationLogEntry.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IOperationLogEntry);

        /** OperationLogEntry id. */
        public id: (number|Long);

        /** OperationLogEntry opTime. */
        public opTime: (number|Long);

        /** OperationLogEntry username. */
        public username: string;

        /** OperationLogEntry role. */
        public role: string;

        /** OperationLogEntry logCategory. */
        public logCategory: string;

        /** OperationLogEntry actionType. */
        public actionType: string;

        /** OperationLogEntry target. */
        public target: string;

        /** OperationLogEntry relatedFile. */
        public relatedFile: string;

        /** OperationLogEntry relatedVersion. */
        public relatedVersion: string;

        /** OperationLogEntry result. */
        public result: string;

        /** OperationLogEntry failReason. */
        public failReason: string;

        /** OperationLogEntry detail. */
        public detail: string;

        /** OperationLogEntry sourceIp. */
        public sourceIp: string;

        /** OperationLogEntry appVersion. */
        public appVersion: string;

        /** OperationLogEntry sessionId. */
        public sessionId: string;

        /** OperationLogEntry requestId. */
        public requestId: string;

        /** OperationLogEntry beforeValue. */
        public beforeValue: string;

        /** OperationLogEntry afterValue. */
        public afterValue: string;

        /**
         * Encodes the specified OperationLogEntry message. Does not implicitly {@link usb_control.OperationLogEntry.verify|verify} messages.
         * @param message OperationLogEntry message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IOperationLogEntry, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified OperationLogEntry message, length delimited. Does not implicitly {@link usb_control.OperationLogEntry.verify|verify} messages.
         * @param message OperationLogEntry message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IOperationLogEntry, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes an OperationLogEntry message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns OperationLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.OperationLogEntry;

        /**
         * Decodes an OperationLogEntry message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns OperationLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.OperationLogEntry;

        /**
         * Creates an OperationLogEntry message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns OperationLogEntry
         */
        public static fromObject(object: { [k: string]: any }): usb_control.OperationLogEntry;

        /**
         * Creates a plain object from an OperationLogEntry message. Also converts values to other types if specified.
         * @param message OperationLogEntry
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.OperationLogEntry, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this OperationLogEntry to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for OperationLogEntry
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspQueryLogs. */
    interface IRspQueryLogs {

        /** RspQueryLogs success */
        success?: (boolean|null);

        /** RspQueryLogs usbAuditEntries */
        usbAuditEntries?: (usb_control.IUsbAuditLogEntry[]|null);

        /** RspQueryLogs malwareEntries */
        malwareEntries?: (usb_control.IMalwareLogEntry[]|null);

        /** RspQueryLogs operationEntries */
        operationEntries?: (usb_control.IOperationLogEntry[]|null);

        /** RspQueryLogs total */
        total?: (number|null);

        /** RspQueryLogs page */
        page?: (number|null);

        /** RspQueryLogs pageSize */
        pageSize?: (number|null);

        /** RspQueryLogs resultCode */
        resultCode?: (number|null);

        /** RspQueryLogs errorMessage */
        errorMessage?: (string|null);
    }

    /** Represents a RspQueryLogs. */
    class RspQueryLogs implements IRspQueryLogs {

        /**
         * Constructs a new RspQueryLogs.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspQueryLogs);

        /** RspQueryLogs success. */
        public success: boolean;

        /** RspQueryLogs usbAuditEntries. */
        public usbAuditEntries: usb_control.IUsbAuditLogEntry[];

        /** RspQueryLogs malwareEntries. */
        public malwareEntries: usb_control.IMalwareLogEntry[];

        /** RspQueryLogs operationEntries. */
        public operationEntries: usb_control.IOperationLogEntry[];

        /** RspQueryLogs total. */
        public total: number;

        /** RspQueryLogs page. */
        public page: number;

        /** RspQueryLogs pageSize. */
        public pageSize: number;

        /** RspQueryLogs resultCode. */
        public resultCode: number;

        /** RspQueryLogs errorMessage. */
        public errorMessage: string;

        /**
         * Encodes the specified RspQueryLogs message. Does not implicitly {@link usb_control.RspQueryLogs.verify|verify} messages.
         * @param message RspQueryLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspQueryLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspQueryLogs message, length delimited. Does not implicitly {@link usb_control.RspQueryLogs.verify|verify} messages.
         * @param message RspQueryLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspQueryLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspQueryLogs message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspQueryLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspQueryLogs;

        /**
         * Decodes a RspQueryLogs message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspQueryLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspQueryLogs;

        /**
         * Creates a RspQueryLogs message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspQueryLogs
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspQueryLogs;

        /**
         * Creates a plain object from a RspQueryLogs message. Also converts values to other types if specified.
         * @param message RspQueryLogs
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspQueryLogs, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspQueryLogs to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspQueryLogs
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdExportLogs. */
    interface ICmdExportLogs {

        /** CmdExportLogs sessionToken */
        sessionToken?: (string|null);

        /** CmdExportLogs logType */
        logType?: (string|null);

        /** CmdExportLogs startTime */
        startTime?: (number|Long|null);

        /** CmdExportLogs endTime */
        endTime?: (number|Long|null);

        /** CmdExportLogs keyword */
        keyword?: (string|null);

        /** CmdExportLogs eventType */
        eventType?: (string|null);
    }

    /** Represents a CmdExportLogs. */
    class CmdExportLogs implements ICmdExportLogs {

        /**
         * Constructs a new CmdExportLogs.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdExportLogs);

        /** CmdExportLogs sessionToken. */
        public sessionToken: string;

        /** CmdExportLogs logType. */
        public logType: string;

        /** CmdExportLogs startTime. */
        public startTime: (number|Long);

        /** CmdExportLogs endTime. */
        public endTime: (number|Long);

        /** CmdExportLogs keyword. */
        public keyword: string;

        /** CmdExportLogs eventType. */
        public eventType: string;

        /**
         * Encodes the specified CmdExportLogs message. Does not implicitly {@link usb_control.CmdExportLogs.verify|verify} messages.
         * @param message CmdExportLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdExportLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdExportLogs message, length delimited. Does not implicitly {@link usb_control.CmdExportLogs.verify|verify} messages.
         * @param message CmdExportLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdExportLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdExportLogs message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdExportLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdExportLogs;

        /**
         * Decodes a CmdExportLogs message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdExportLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdExportLogs;

        /**
         * Creates a CmdExportLogs message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdExportLogs
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdExportLogs;

        /**
         * Creates a plain object from a CmdExportLogs message. Also converts values to other types if specified.
         * @param message CmdExportLogs
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdExportLogs, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdExportLogs to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdExportLogs
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspExportLogs. */
    interface IRspExportLogs {

        /** RspExportLogs success */
        success?: (boolean|null);

        /** RspExportLogs zipData */
        zipData?: (Uint8Array|null);

        /** RspExportLogs suggestedFilename */
        suggestedFilename?: (string|null);

        /** RspExportLogs resultCode */
        resultCode?: (number|null);

        /** RspExportLogs errorMessage */
        errorMessage?: (string|null);
    }

    /** Represents a RspExportLogs. */
    class RspExportLogs implements IRspExportLogs {

        /**
         * Constructs a new RspExportLogs.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspExportLogs);

        /** RspExportLogs success. */
        public success: boolean;

        /** RspExportLogs zipData. */
        public zipData: Uint8Array;

        /** RspExportLogs suggestedFilename. */
        public suggestedFilename: string;

        /** RspExportLogs resultCode. */
        public resultCode: number;

        /** RspExportLogs errorMessage. */
        public errorMessage: string;

        /**
         * Encodes the specified RspExportLogs message. Does not implicitly {@link usb_control.RspExportLogs.verify|verify} messages.
         * @param message RspExportLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspExportLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspExportLogs message, length delimited. Does not implicitly {@link usb_control.RspExportLogs.verify|verify} messages.
         * @param message RspExportLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspExportLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspExportLogs message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspExportLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspExportLogs;

        /**
         * Decodes a RspExportLogs message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspExportLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspExportLogs;

        /**
         * Creates a RspExportLogs message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspExportLogs
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspExportLogs;

        /**
         * Creates a plain object from a RspExportLogs message. Also converts values to other types if specified.
         * @param message RspExportLogs
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspExportLogs, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspExportLogs to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspExportLogs
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdDeleteLogs. */
    interface ICmdDeleteLogs {

        /** CmdDeleteLogs sessionToken */
        sessionToken?: (string|null);

        /** CmdDeleteLogs logType */
        logType?: (string|null);

        /** CmdDeleteLogs startTime */
        startTime?: (number|Long|null);

        /** CmdDeleteLogs endTime */
        endTime?: (number|Long|null);
    }

    /** Represents a CmdDeleteLogs. */
    class CmdDeleteLogs implements ICmdDeleteLogs {

        /**
         * Constructs a new CmdDeleteLogs.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdDeleteLogs);

        /** CmdDeleteLogs sessionToken. */
        public sessionToken: string;

        /** CmdDeleteLogs logType. */
        public logType: string;

        /** CmdDeleteLogs startTime. */
        public startTime: (number|Long);

        /** CmdDeleteLogs endTime. */
        public endTime: (number|Long);

        /**
         * Encodes the specified CmdDeleteLogs message. Does not implicitly {@link usb_control.CmdDeleteLogs.verify|verify} messages.
         * @param message CmdDeleteLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdDeleteLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdDeleteLogs message, length delimited. Does not implicitly {@link usb_control.CmdDeleteLogs.verify|verify} messages.
         * @param message CmdDeleteLogs message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdDeleteLogs, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdDeleteLogs message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdDeleteLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdDeleteLogs;

        /**
         * Decodes a CmdDeleteLogs message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdDeleteLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdDeleteLogs;

        /**
         * Creates a CmdDeleteLogs message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdDeleteLogs
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdDeleteLogs;

        /**
         * Creates a plain object from a CmdDeleteLogs message. Also converts values to other types if specified.
         * @param message CmdDeleteLogs
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdDeleteLogs, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdDeleteLogs to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdDeleteLogs
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdGetSystemInfo. */
    interface ICmdGetSystemInfo {

        /** CmdGetSystemInfo sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdGetSystemInfo. */
    class CmdGetSystemInfo implements ICmdGetSystemInfo {

        /**
         * Constructs a new CmdGetSystemInfo.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdGetSystemInfo);

        /** CmdGetSystemInfo sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdGetSystemInfo message. Does not implicitly {@link usb_control.CmdGetSystemInfo.verify|verify} messages.
         * @param message CmdGetSystemInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdGetSystemInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdGetSystemInfo message, length delimited. Does not implicitly {@link usb_control.CmdGetSystemInfo.verify|verify} messages.
         * @param message CmdGetSystemInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdGetSystemInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdGetSystemInfo message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdGetSystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdGetSystemInfo;

        /**
         * Decodes a CmdGetSystemInfo message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdGetSystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdGetSystemInfo;

        /**
         * Creates a CmdGetSystemInfo message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdGetSystemInfo
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdGetSystemInfo;

        /**
         * Creates a plain object from a CmdGetSystemInfo message. Also converts values to other types if specified.
         * @param message CmdGetSystemInfo
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdGetSystemInfo, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdGetSystemInfo to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdGetSystemInfo
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspSystemInfo. */
    interface IRspSystemInfo {

        /** RspSystemInfo systemVersion */
        systemVersion?: (string|null);

        /** RspSystemInfo virusDbVersion */
        virusDbVersion?: (string|null);

        /** RspSystemInfo authorized */
        authorized?: (boolean|null);

        /** RspSystemInfo authExpireTime */
        authExpireTime?: (number|Long|null);

        /** RspSystemInfo deviceDescription */
        deviceDescription?: (string|null);

        /** RspSystemInfo virusDbUpdatedAt */
        virusDbUpdatedAt?: (number|Long|null);

        /** RspSystemInfo authStatus */
        authStatus?: (string|null);
    }

    /** Represents a RspSystemInfo. */
    class RspSystemInfo implements IRspSystemInfo {

        /**
         * Constructs a new RspSystemInfo.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspSystemInfo);

        /** RspSystemInfo systemVersion. */
        public systemVersion: string;

        /** RspSystemInfo virusDbVersion. */
        public virusDbVersion: string;

        /** RspSystemInfo authorized. */
        public authorized: boolean;

        /** RspSystemInfo authExpireTime. */
        public authExpireTime: (number|Long);

        /** RspSystemInfo deviceDescription. */
        public deviceDescription: string;

        /** RspSystemInfo virusDbUpdatedAt. */
        public virusDbUpdatedAt: (number|Long);

        /** RspSystemInfo authStatus. */
        public authStatus: string;

        /**
         * Encodes the specified RspSystemInfo message. Does not implicitly {@link usb_control.RspSystemInfo.verify|verify} messages.
         * @param message RspSystemInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspSystemInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspSystemInfo message, length delimited. Does not implicitly {@link usb_control.RspSystemInfo.verify|verify} messages.
         * @param message RspSystemInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspSystemInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspSystemInfo message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspSystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspSystemInfo;

        /**
         * Decodes a RspSystemInfo message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspSystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspSystemInfo;

        /**
         * Creates a RspSystemInfo message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspSystemInfo
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspSystemInfo;

        /**
         * Creates a plain object from a RspSystemInfo message. Also converts values to other types if specified.
         * @param message RspSystemInfo
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspSystemInfo, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspSystemInfo to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspSystemInfo
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdUploadSystemUpgrade. */
    interface ICmdUploadSystemUpgrade {

        /** CmdUploadSystemUpgrade sessionToken */
        sessionToken?: (string|null);

        /** CmdUploadSystemUpgrade upgradeData */
        upgradeData?: (Uint8Array|null);

        /** CmdUploadSystemUpgrade targetVersion */
        targetVersion?: (string|null);

        /** CmdUploadSystemUpgrade sha256Checksum */
        sha256Checksum?: (string|null);
    }

    /** Represents a CmdUploadSystemUpgrade. */
    class CmdUploadSystemUpgrade implements ICmdUploadSystemUpgrade {

        /**
         * Constructs a new CmdUploadSystemUpgrade.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdUploadSystemUpgrade);

        /** CmdUploadSystemUpgrade sessionToken. */
        public sessionToken: string;

        /** CmdUploadSystemUpgrade upgradeData. */
        public upgradeData: Uint8Array;

        /** CmdUploadSystemUpgrade targetVersion. */
        public targetVersion: string;

        /** CmdUploadSystemUpgrade sha256Checksum. */
        public sha256Checksum: string;

        /**
         * Encodes the specified CmdUploadSystemUpgrade message. Does not implicitly {@link usb_control.CmdUploadSystemUpgrade.verify|verify} messages.
         * @param message CmdUploadSystemUpgrade message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdUploadSystemUpgrade, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdUploadSystemUpgrade message, length delimited. Does not implicitly {@link usb_control.CmdUploadSystemUpgrade.verify|verify} messages.
         * @param message CmdUploadSystemUpgrade message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdUploadSystemUpgrade, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdUploadSystemUpgrade message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdUploadSystemUpgrade
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdUploadSystemUpgrade;

        /**
         * Decodes a CmdUploadSystemUpgrade message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdUploadSystemUpgrade
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdUploadSystemUpgrade;

        /**
         * Creates a CmdUploadSystemUpgrade message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdUploadSystemUpgrade
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdUploadSystemUpgrade;

        /**
         * Creates a plain object from a CmdUploadSystemUpgrade message. Also converts values to other types if specified.
         * @param message CmdUploadSystemUpgrade
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdUploadSystemUpgrade, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdUploadSystemUpgrade to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdUploadSystemUpgrade
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdUploadVirusdbUpgrade. */
    interface ICmdUploadVirusdbUpgrade {

        /** CmdUploadVirusdbUpgrade sessionToken */
        sessionToken?: (string|null);

        /** CmdUploadVirusdbUpgrade upgradeData */
        upgradeData?: (Uint8Array|null);

        /** CmdUploadVirusdbUpgrade targetVersion */
        targetVersion?: (string|null);

        /** CmdUploadVirusdbUpgrade sha256Checksum */
        sha256Checksum?: (string|null);
    }

    /** Represents a CmdUploadVirusdbUpgrade. */
    class CmdUploadVirusdbUpgrade implements ICmdUploadVirusdbUpgrade {

        /**
         * Constructs a new CmdUploadVirusdbUpgrade.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdUploadVirusdbUpgrade);

        /** CmdUploadVirusdbUpgrade sessionToken. */
        public sessionToken: string;

        /** CmdUploadVirusdbUpgrade upgradeData. */
        public upgradeData: Uint8Array;

        /** CmdUploadVirusdbUpgrade targetVersion. */
        public targetVersion: string;

        /** CmdUploadVirusdbUpgrade sha256Checksum. */
        public sha256Checksum: string;

        /**
         * Encodes the specified CmdUploadVirusdbUpgrade message. Does not implicitly {@link usb_control.CmdUploadVirusdbUpgrade.verify|verify} messages.
         * @param message CmdUploadVirusdbUpgrade message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdUploadVirusdbUpgrade, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdUploadVirusdbUpgrade message, length delimited. Does not implicitly {@link usb_control.CmdUploadVirusdbUpgrade.verify|verify} messages.
         * @param message CmdUploadVirusdbUpgrade message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdUploadVirusdbUpgrade, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdUploadVirusdbUpgrade message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdUploadVirusdbUpgrade
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdUploadVirusdbUpgrade;

        /**
         * Decodes a CmdUploadVirusdbUpgrade message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdUploadVirusdbUpgrade
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdUploadVirusdbUpgrade;

        /**
         * Creates a CmdUploadVirusdbUpgrade message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdUploadVirusdbUpgrade
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdUploadVirusdbUpgrade;

        /**
         * Creates a plain object from a CmdUploadVirusdbUpgrade message. Also converts values to other types if specified.
         * @param message CmdUploadVirusdbUpgrade
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdUploadVirusdbUpgrade, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdUploadVirusdbUpgrade to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdUploadVirusdbUpgrade
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdUpdateDeviceDesc. */
    interface ICmdUpdateDeviceDesc {

        /** CmdUpdateDeviceDesc sessionToken */
        sessionToken?: (string|null);

        /** CmdUpdateDeviceDesc description */
        description?: (string|null);
    }

    /** Represents a CmdUpdateDeviceDesc. */
    class CmdUpdateDeviceDesc implements ICmdUpdateDeviceDesc {

        /**
         * Constructs a new CmdUpdateDeviceDesc.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdUpdateDeviceDesc);

        /** CmdUpdateDeviceDesc sessionToken. */
        public sessionToken: string;

        /** CmdUpdateDeviceDesc description. */
        public description: string;

        /**
         * Encodes the specified CmdUpdateDeviceDesc message. Does not implicitly {@link usb_control.CmdUpdateDeviceDesc.verify|verify} messages.
         * @param message CmdUpdateDeviceDesc message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdUpdateDeviceDesc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdUpdateDeviceDesc message, length delimited. Does not implicitly {@link usb_control.CmdUpdateDeviceDesc.verify|verify} messages.
         * @param message CmdUpdateDeviceDesc message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdUpdateDeviceDesc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdUpdateDeviceDesc message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdUpdateDeviceDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdUpdateDeviceDesc;

        /**
         * Decodes a CmdUpdateDeviceDesc message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdUpdateDeviceDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdUpdateDeviceDesc;

        /**
         * Creates a CmdUpdateDeviceDesc message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdUpdateDeviceDesc
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdUpdateDeviceDesc;

        /**
         * Creates a plain object from a CmdUpdateDeviceDesc message. Also converts values to other types if specified.
         * @param message CmdUpdateDeviceDesc
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdUpdateDeviceDesc, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdUpdateDeviceDesc to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdUpdateDeviceDesc
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a UserItem. */
    interface IUserItem {

        /** UserItem username */
        username?: (string|null);

        /** UserItem role */
        role?: (string|null);

        /** UserItem status */
        status?: (string|null);

        /** UserItem isBuiltin */
        isBuiltin?: (boolean|null);

        /** UserItem createdAt */
        createdAt?: (number|Long|null);
    }

    /** Represents a UserItem. */
    class UserItem implements IUserItem {

        /**
         * Constructs a new UserItem.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IUserItem);

        /** UserItem username. */
        public username: string;

        /** UserItem role. */
        public role: string;

        /** UserItem status. */
        public status: string;

        /** UserItem isBuiltin. */
        public isBuiltin: boolean;

        /** UserItem createdAt. */
        public createdAt: (number|Long);

        /**
         * Encodes the specified UserItem message. Does not implicitly {@link usb_control.UserItem.verify|verify} messages.
         * @param message UserItem message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IUserItem, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified UserItem message, length delimited. Does not implicitly {@link usb_control.UserItem.verify|verify} messages.
         * @param message UserItem message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IUserItem, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a UserItem message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns UserItem
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.UserItem;

        /**
         * Decodes a UserItem message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns UserItem
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.UserItem;

        /**
         * Creates a UserItem message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns UserItem
         */
        public static fromObject(object: { [k: string]: any }): usb_control.UserItem;

        /**
         * Creates a plain object from a UserItem message. Also converts values to other types if specified.
         * @param message UserItem
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.UserItem, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this UserItem to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for UserItem
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdListUsers. */
    interface ICmdListUsers {

        /** CmdListUsers sessionToken */
        sessionToken?: (string|null);
    }

    /** Represents a CmdListUsers. */
    class CmdListUsers implements ICmdListUsers {

        /**
         * Constructs a new CmdListUsers.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdListUsers);

        /** CmdListUsers sessionToken. */
        public sessionToken: string;

        /**
         * Encodes the specified CmdListUsers message. Does not implicitly {@link usb_control.CmdListUsers.verify|verify} messages.
         * @param message CmdListUsers message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdListUsers, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdListUsers message, length delimited. Does not implicitly {@link usb_control.CmdListUsers.verify|verify} messages.
         * @param message CmdListUsers message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdListUsers, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdListUsers message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdListUsers
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdListUsers;

        /**
         * Decodes a CmdListUsers message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdListUsers
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdListUsers;

        /**
         * Creates a CmdListUsers message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdListUsers
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdListUsers;

        /**
         * Creates a plain object from a CmdListUsers message. Also converts values to other types if specified.
         * @param message CmdListUsers
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdListUsers, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdListUsers to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdListUsers
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspListUsers. */
    interface IRspListUsers {

        /** RspListUsers users */
        users?: (usb_control.IUserItem[]|null);
    }

    /** Represents a RspListUsers. */
    class RspListUsers implements IRspListUsers {

        /**
         * Constructs a new RspListUsers.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspListUsers);

        /** RspListUsers users. */
        public users: usb_control.IUserItem[];

        /**
         * Encodes the specified RspListUsers message. Does not implicitly {@link usb_control.RspListUsers.verify|verify} messages.
         * @param message RspListUsers message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspListUsers, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspListUsers message, length delimited. Does not implicitly {@link usb_control.RspListUsers.verify|verify} messages.
         * @param message RspListUsers message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspListUsers, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspListUsers message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspListUsers
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspListUsers;

        /**
         * Decodes a RspListUsers message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspListUsers
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspListUsers;

        /**
         * Creates a RspListUsers message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspListUsers
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspListUsers;

        /**
         * Creates a plain object from a RspListUsers message. Also converts values to other types if specified.
         * @param message RspListUsers
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspListUsers, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspListUsers to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspListUsers
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdCreateUser. */
    interface ICmdCreateUser {

        /** CmdCreateUser sessionToken */
        sessionToken?: (string|null);

        /** CmdCreateUser username */
        username?: (string|null);

        /** CmdCreateUser role */
        role?: (string|null);

        /** CmdCreateUser password */
        password?: (string|null);

        /** CmdCreateUser confirmPassword */
        confirmPassword?: (string|null);
    }

    /** Represents a CmdCreateUser. */
    class CmdCreateUser implements ICmdCreateUser {

        /**
         * Constructs a new CmdCreateUser.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdCreateUser);

        /** CmdCreateUser sessionToken. */
        public sessionToken: string;

        /** CmdCreateUser username. */
        public username: string;

        /** CmdCreateUser role. */
        public role: string;

        /** CmdCreateUser password. */
        public password: string;

        /** CmdCreateUser confirmPassword. */
        public confirmPassword: string;

        /**
         * Encodes the specified CmdCreateUser message. Does not implicitly {@link usb_control.CmdCreateUser.verify|verify} messages.
         * @param message CmdCreateUser message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdCreateUser, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdCreateUser message, length delimited. Does not implicitly {@link usb_control.CmdCreateUser.verify|verify} messages.
         * @param message CmdCreateUser message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdCreateUser, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdCreateUser message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdCreateUser
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdCreateUser;

        /**
         * Decodes a CmdCreateUser message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdCreateUser
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdCreateUser;

        /**
         * Creates a CmdCreateUser message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdCreateUser
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdCreateUser;

        /**
         * Creates a plain object from a CmdCreateUser message. Also converts values to other types if specified.
         * @param message CmdCreateUser
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdCreateUser, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdCreateUser to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdCreateUser
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdDeleteUser. */
    interface ICmdDeleteUser {

        /** CmdDeleteUser sessionToken */
        sessionToken?: (string|null);

        /** CmdDeleteUser username */
        username?: (string|null);
    }

    /** Represents a CmdDeleteUser. */
    class CmdDeleteUser implements ICmdDeleteUser {

        /**
         * Constructs a new CmdDeleteUser.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdDeleteUser);

        /** CmdDeleteUser sessionToken. */
        public sessionToken: string;

        /** CmdDeleteUser username. */
        public username: string;

        /**
         * Encodes the specified CmdDeleteUser message. Does not implicitly {@link usb_control.CmdDeleteUser.verify|verify} messages.
         * @param message CmdDeleteUser message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdDeleteUser, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdDeleteUser message, length delimited. Does not implicitly {@link usb_control.CmdDeleteUser.verify|verify} messages.
         * @param message CmdDeleteUser message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdDeleteUser, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdDeleteUser message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdDeleteUser
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdDeleteUser;

        /**
         * Decodes a CmdDeleteUser message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdDeleteUser
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdDeleteUser;

        /**
         * Creates a CmdDeleteUser message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdDeleteUser
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdDeleteUser;

        /**
         * Creates a plain object from a CmdDeleteUser message. Also converts values to other types if specified.
         * @param message CmdDeleteUser
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdDeleteUser, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdDeleteUser to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdDeleteUser
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdResetPassword. */
    interface ICmdResetPassword {

        /** CmdResetPassword sessionToken */
        sessionToken?: (string|null);

        /** CmdResetPassword username */
        username?: (string|null);

        /** CmdResetPassword newPassword */
        newPassword?: (string|null);

        /** CmdResetPassword confirmPassword */
        confirmPassword?: (string|null);
    }

    /** Represents a CmdResetPassword. */
    class CmdResetPassword implements ICmdResetPassword {

        /**
         * Constructs a new CmdResetPassword.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdResetPassword);

        /** CmdResetPassword sessionToken. */
        public sessionToken: string;

        /** CmdResetPassword username. */
        public username: string;

        /** CmdResetPassword newPassword. */
        public newPassword: string;

        /** CmdResetPassword confirmPassword. */
        public confirmPassword: string;

        /**
         * Encodes the specified CmdResetPassword message. Does not implicitly {@link usb_control.CmdResetPassword.verify|verify} messages.
         * @param message CmdResetPassword message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdResetPassword, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdResetPassword message, length delimited. Does not implicitly {@link usb_control.CmdResetPassword.verify|verify} messages.
         * @param message CmdResetPassword message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdResetPassword, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdResetPassword message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdResetPassword
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdResetPassword;

        /**
         * Decodes a CmdResetPassword message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdResetPassword
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdResetPassword;

        /**
         * Creates a CmdResetPassword message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdResetPassword
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdResetPassword;

        /**
         * Creates a plain object from a CmdResetPassword message. Also converts values to other types if specified.
         * @param message CmdResetPassword
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdResetPassword, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdResetPassword to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdResetPassword
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdChangePassword. */
    interface ICmdChangePassword {

        /** CmdChangePassword sessionToken */
        sessionToken?: (string|null);

        /** CmdChangePassword oldPassword */
        oldPassword?: (string|null);

        /** CmdChangePassword newPassword */
        newPassword?: (string|null);

        /** CmdChangePassword confirmPassword */
        confirmPassword?: (string|null);
    }

    /** Represents a CmdChangePassword. */
    class CmdChangePassword implements ICmdChangePassword {

        /**
         * Constructs a new CmdChangePassword.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdChangePassword);

        /** CmdChangePassword sessionToken. */
        public sessionToken: string;

        /** CmdChangePassword oldPassword. */
        public oldPassword: string;

        /** CmdChangePassword newPassword. */
        public newPassword: string;

        /** CmdChangePassword confirmPassword. */
        public confirmPassword: string;

        /**
         * Encodes the specified CmdChangePassword message. Does not implicitly {@link usb_control.CmdChangePassword.verify|verify} messages.
         * @param message CmdChangePassword message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdChangePassword, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdChangePassword message, length delimited. Does not implicitly {@link usb_control.CmdChangePassword.verify|verify} messages.
         * @param message CmdChangePassword message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdChangePassword, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdChangePassword message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdChangePassword
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdChangePassword;

        /**
         * Decodes a CmdChangePassword message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdChangePassword
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdChangePassword;

        /**
         * Creates a CmdChangePassword message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdChangePassword
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdChangePassword;

        /**
         * Creates a plain object from a CmdChangePassword message. Also converts values to other types if specified.
         * @param message CmdChangePassword
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdChangePassword, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdChangePassword to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdChangePassword
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspCommon. */
    interface IRspCommon {

        /** RspCommon success */
        success?: (boolean|null);

        /** RspCommon resultCode */
        resultCode?: (number|null);

        /** RspCommon errorMessage */
        errorMessage?: (string|null);
    }

    /** Represents a RspCommon. */
    class RspCommon implements IRspCommon {

        /**
         * Constructs a new RspCommon.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspCommon);

        /** RspCommon success. */
        public success: boolean;

        /** RspCommon resultCode. */
        public resultCode: number;

        /** RspCommon errorMessage. */
        public errorMessage: string;

        /**
         * Encodes the specified RspCommon message. Does not implicitly {@link usb_control.RspCommon.verify|verify} messages.
         * @param message RspCommon message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspCommon, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspCommon message, length delimited. Does not implicitly {@link usb_control.RspCommon.verify|verify} messages.
         * @param message RspCommon message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspCommon, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspCommon message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspCommon
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspCommon;

        /**
         * Decodes a RspCommon message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspCommon
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspCommon;

        /**
         * Creates a RspCommon message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspCommon
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspCommon;

        /**
         * Creates a plain object from a RspCommon message. Also converts values to other types if specified.
         * @param message RspCommon
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspCommon, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspCommon to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspCommon
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CmdHeartbeat. */
    interface ICmdHeartbeat {
    }

    /** Represents a CmdHeartbeat. */
    class CmdHeartbeat implements ICmdHeartbeat {

        /**
         * Constructs a new CmdHeartbeat.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.ICmdHeartbeat);

        /**
         * Encodes the specified CmdHeartbeat message. Does not implicitly {@link usb_control.CmdHeartbeat.verify|verify} messages.
         * @param message CmdHeartbeat message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.ICmdHeartbeat, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CmdHeartbeat message, length delimited. Does not implicitly {@link usb_control.CmdHeartbeat.verify|verify} messages.
         * @param message CmdHeartbeat message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.ICmdHeartbeat, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CmdHeartbeat message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CmdHeartbeat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.CmdHeartbeat;

        /**
         * Decodes a CmdHeartbeat message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CmdHeartbeat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.CmdHeartbeat;

        /**
         * Creates a CmdHeartbeat message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CmdHeartbeat
         */
        public static fromObject(object: { [k: string]: any }): usb_control.CmdHeartbeat;

        /**
         * Creates a plain object from a CmdHeartbeat message. Also converts values to other types if specified.
         * @param message CmdHeartbeat
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.CmdHeartbeat, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CmdHeartbeat to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CmdHeartbeat
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RspHeartbeat. */
    interface IRspHeartbeat {
    }

    /** Represents a RspHeartbeat. */
    class RspHeartbeat implements IRspHeartbeat {

        /**
         * Constructs a new RspHeartbeat.
         * @param [properties] Properties to set
         */
        constructor(properties?: usb_control.IRspHeartbeat);

        /**
         * Encodes the specified RspHeartbeat message. Does not implicitly {@link usb_control.RspHeartbeat.verify|verify} messages.
         * @param message RspHeartbeat message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: usb_control.IRspHeartbeat, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RspHeartbeat message, length delimited. Does not implicitly {@link usb_control.RspHeartbeat.verify|verify} messages.
         * @param message RspHeartbeat message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: usb_control.IRspHeartbeat, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RspHeartbeat message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RspHeartbeat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): usb_control.RspHeartbeat;

        /**
         * Decodes a RspHeartbeat message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RspHeartbeat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): usb_control.RspHeartbeat;

        /**
         * Creates a RspHeartbeat message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RspHeartbeat
         */
        public static fromObject(object: { [k: string]: any }): usb_control.RspHeartbeat;

        /**
         * Creates a plain object from a RspHeartbeat message. Also converts values to other types if specified.
         * @param message RspHeartbeat
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: usb_control.RspHeartbeat, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RspHeartbeat to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RspHeartbeat
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }
}
