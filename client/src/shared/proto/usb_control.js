/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
import * as $protobuf from "protobufjs/minimal";

// Common aliases
const $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

// Exported root namespace
const $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});

export const usb_control = $root.usb_control = (() => {

    /**
     * Namespace usb_control.
     * @exports usb_control
     * @namespace
     */
    const usb_control = {};

    usb_control.CmdLogin = (function() {

        /**
         * Properties of a CmdLogin.
         * @memberof usb_control
         * @interface ICmdLogin
         * @property {string|null} [username] CmdLogin username
         * @property {string|null} [password] CmdLogin password
         */

        /**
         * Constructs a new CmdLogin.
         * @memberof usb_control
         * @classdesc Represents a CmdLogin.
         * @implements ICmdLogin
         * @constructor
         * @param {usb_control.ICmdLogin=} [properties] Properties to set
         */
        function CmdLogin(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdLogin username.
         * @member {string} username
         * @memberof usb_control.CmdLogin
         * @instance
         */
        CmdLogin.prototype.username = "";

        /**
         * CmdLogin password.
         * @member {string} password
         * @memberof usb_control.CmdLogin
         * @instance
         */
        CmdLogin.prototype.password = "";

        /**
         * Encodes the specified CmdLogin message. Does not implicitly {@link usb_control.CmdLogin.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdLogin
         * @static
         * @param {usb_control.ICmdLogin} message CmdLogin message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdLogin.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.username);
            if (message.password != null && Object.hasOwnProperty.call(message, "password"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.password);
            return writer;
        };

        /**
         * Encodes the specified CmdLogin message, length delimited. Does not implicitly {@link usb_control.CmdLogin.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdLogin
         * @static
         * @param {usb_control.ICmdLogin} message CmdLogin message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdLogin.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdLogin message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdLogin
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdLogin} CmdLogin
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdLogin.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdLogin();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.username = reader.string();
                        break;
                    }
                case 2: {
                        message.password = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdLogin message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdLogin
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdLogin} CmdLogin
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdLogin.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdLogin message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdLogin
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdLogin} CmdLogin
         */
        CmdLogin.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdLogin)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdLogin: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdLogin();
            if (object.username != null)
                message.username = String(object.username);
            if (object.password != null)
                message.password = String(object.password);
            return message;
        };

        /**
         * Creates a plain object from a CmdLogin message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdLogin
         * @static
         * @param {usb_control.CmdLogin} message CmdLogin
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdLogin.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.username = "";
                object.password = "";
            }
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                object.username = message.username;
            if (message.password != null && Object.hasOwnProperty.call(message, "password"))
                object.password = message.password;
            return object;
        };

        /**
         * Converts this CmdLogin to JSON.
         * @function toJSON
         * @memberof usb_control.CmdLogin
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdLogin.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdLogin
         * @function getTypeUrl
         * @memberof usb_control.CmdLogin
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdLogin.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdLogin";
        };

        return CmdLogin;
    })();

    usb_control.RspLogin = (function() {

        /**
         * Properties of a RspLogin.
         * @memberof usb_control
         * @interface IRspLogin
         * @property {boolean|null} [success] RspLogin success
         * @property {string|null} [sessionToken] RspLogin sessionToken
         * @property {string|null} [username] RspLogin username
         * @property {string|null} [role] RspLogin role
         * @property {boolean|null} [authorized] RspLogin authorized
         * @property {number|Long|null} [authExpireTime] RspLogin authExpireTime
         * @property {string|null} [deviceDescription] RspLogin deviceDescription
         * @property {number|null} [resultCode] RspLogin resultCode
         * @property {string|null} [errorMessage] RspLogin errorMessage
         * @property {string|null} [authStatus] RspLogin authStatus
         */

        /**
         * Constructs a new RspLogin.
         * @memberof usb_control
         * @classdesc Represents a RspLogin.
         * @implements IRspLogin
         * @constructor
         * @param {usb_control.IRspLogin=} [properties] Properties to set
         */
        function RspLogin(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspLogin success.
         * @member {boolean} success
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.success = false;

        /**
         * RspLogin sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.sessionToken = "";

        /**
         * RspLogin username.
         * @member {string} username
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.username = "";

        /**
         * RspLogin role.
         * @member {string} role
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.role = "";

        /**
         * RspLogin authorized.
         * @member {boolean} authorized
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.authorized = false;

        /**
         * RspLogin authExpireTime.
         * @member {number|Long} authExpireTime
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.authExpireTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * RspLogin deviceDescription.
         * @member {string} deviceDescription
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.deviceDescription = "";

        /**
         * RspLogin resultCode.
         * @member {number} resultCode
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.resultCode = 0;

        /**
         * RspLogin errorMessage.
         * @member {string} errorMessage
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.errorMessage = "";

        /**
         * RspLogin authStatus.
         * @member {string} authStatus
         * @memberof usb_control.RspLogin
         * @instance
         */
        RspLogin.prototype.authStatus = "";

        /**
         * Encodes the specified RspLogin message. Does not implicitly {@link usb_control.RspLogin.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspLogin
         * @static
         * @param {usb_control.IRspLogin} message RspLogin message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspLogin.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.sessionToken);
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.username);
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.role);
            if (message.authorized != null && Object.hasOwnProperty.call(message, "authorized"))
                writer.uint32(/* id 5, wireType 0 =*/40).bool(message.authorized);
            if (message.authExpireTime != null && Object.hasOwnProperty.call(message, "authExpireTime"))
                writer.uint32(/* id 6, wireType 0 =*/48).int64(message.authExpireTime);
            if (message.deviceDescription != null && Object.hasOwnProperty.call(message, "deviceDescription"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.deviceDescription);
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                writer.uint32(/* id 8, wireType 0 =*/64).int32(message.resultCode);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 9, wireType 2 =*/74).string(message.errorMessage);
            if (message.authStatus != null && Object.hasOwnProperty.call(message, "authStatus"))
                writer.uint32(/* id 10, wireType 2 =*/82).string(message.authStatus);
            return writer;
        };

        /**
         * Encodes the specified RspLogin message, length delimited. Does not implicitly {@link usb_control.RspLogin.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspLogin
         * @static
         * @param {usb_control.IRspLogin} message RspLogin message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspLogin.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspLogin message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspLogin
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspLogin} RspLogin
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspLogin.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspLogin();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.success = reader.bool();
                        break;
                    }
                case 2: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 3: {
                        message.username = reader.string();
                        break;
                    }
                case 4: {
                        message.role = reader.string();
                        break;
                    }
                case 5: {
                        message.authorized = reader.bool();
                        break;
                    }
                case 6: {
                        message.authExpireTime = reader.int64();
                        break;
                    }
                case 7: {
                        message.deviceDescription = reader.string();
                        break;
                    }
                case 8: {
                        message.resultCode = reader.int32();
                        break;
                    }
                case 9: {
                        message.errorMessage = reader.string();
                        break;
                    }
                case 10: {
                        message.authStatus = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspLogin message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspLogin
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspLogin} RspLogin
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspLogin.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspLogin message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspLogin
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspLogin} RspLogin
         */
        RspLogin.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspLogin)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspLogin: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspLogin();
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.username != null)
                message.username = String(object.username);
            if (object.role != null)
                message.role = String(object.role);
            if (object.authorized != null)
                message.authorized = Boolean(object.authorized);
            if (object.authExpireTime != null)
                if ($util.Long)
                    message.authExpireTime = $util.Long.fromValue(object.authExpireTime, false);
                else if (typeof object.authExpireTime === "string")
                    message.authExpireTime = parseInt(object.authExpireTime, 10);
                else if (typeof object.authExpireTime === "number")
                    message.authExpireTime = object.authExpireTime;
                else if (typeof object.authExpireTime === "object")
                    message.authExpireTime = new $util.LongBits(object.authExpireTime.low >>> 0, object.authExpireTime.high >>> 0).toNumber();
            if (object.deviceDescription != null)
                message.deviceDescription = String(object.deviceDescription);
            if (object.resultCode != null)
                message.resultCode = object.resultCode | 0;
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            if (object.authStatus != null)
                message.authStatus = String(object.authStatus);
            return message;
        };

        /**
         * Creates a plain object from a RspLogin message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspLogin
         * @static
         * @param {usb_control.RspLogin} message RspLogin
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspLogin.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.success = false;
                object.sessionToken = "";
                object.username = "";
                object.role = "";
                object.authorized = false;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.authExpireTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.authExpireTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.deviceDescription = "";
                object.resultCode = 0;
                object.errorMessage = "";
                object.authStatus = "";
            }
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                object.success = message.success;
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                object.username = message.username;
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                object.role = message.role;
            if (message.authorized != null && Object.hasOwnProperty.call(message, "authorized"))
                object.authorized = message.authorized;
            if (message.authExpireTime != null && Object.hasOwnProperty.call(message, "authExpireTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.authExpireTime = typeof message.authExpireTime === "number" ? BigInt(message.authExpireTime) : $util.Long.fromBits(message.authExpireTime.low >>> 0, message.authExpireTime.high >>> 0, false).toBigInt();
                else if (typeof message.authExpireTime === "number")
                    object.authExpireTime = options.longs === String ? String(message.authExpireTime) : message.authExpireTime;
                else
                    object.authExpireTime = options.longs === String ? $util.Long.prototype.toString.call(message.authExpireTime) : options.longs === Number ? new $util.LongBits(message.authExpireTime.low >>> 0, message.authExpireTime.high >>> 0).toNumber() : message.authExpireTime;
            if (message.deviceDescription != null && Object.hasOwnProperty.call(message, "deviceDescription"))
                object.deviceDescription = message.deviceDescription;
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                object.resultCode = message.resultCode;
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                object.errorMessage = message.errorMessage;
            if (message.authStatus != null && Object.hasOwnProperty.call(message, "authStatus"))
                object.authStatus = message.authStatus;
            return object;
        };

        /**
         * Converts this RspLogin to JSON.
         * @function toJSON
         * @memberof usb_control.RspLogin
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspLogin.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspLogin
         * @function getTypeUrl
         * @memberof usb_control.RspLogin
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspLogin.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspLogin";
        };

        return RspLogin;
    })();

    usb_control.CmdAuthStatusQuery = (function() {

        /**
         * Properties of a CmdAuthStatusQuery.
         * @memberof usb_control
         * @interface ICmdAuthStatusQuery
         * @property {string|null} [sessionToken] CmdAuthStatusQuery sessionToken
         */

        /**
         * Constructs a new CmdAuthStatusQuery.
         * @memberof usb_control
         * @classdesc Represents a CmdAuthStatusQuery.
         * @implements ICmdAuthStatusQuery
         * @constructor
         * @param {usb_control.ICmdAuthStatusQuery=} [properties] Properties to set
         */
        function CmdAuthStatusQuery(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdAuthStatusQuery sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdAuthStatusQuery
         * @instance
         */
        CmdAuthStatusQuery.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdAuthStatusQuery message. Does not implicitly {@link usb_control.CmdAuthStatusQuery.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdAuthStatusQuery
         * @static
         * @param {usb_control.ICmdAuthStatusQuery} message CmdAuthStatusQuery message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdAuthStatusQuery.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdAuthStatusQuery message, length delimited. Does not implicitly {@link usb_control.CmdAuthStatusQuery.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdAuthStatusQuery
         * @static
         * @param {usb_control.ICmdAuthStatusQuery} message CmdAuthStatusQuery message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdAuthStatusQuery.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdAuthStatusQuery message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdAuthStatusQuery
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdAuthStatusQuery} CmdAuthStatusQuery
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdAuthStatusQuery.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdAuthStatusQuery();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdAuthStatusQuery message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdAuthStatusQuery
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdAuthStatusQuery} CmdAuthStatusQuery
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdAuthStatusQuery.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdAuthStatusQuery message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdAuthStatusQuery
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdAuthStatusQuery} CmdAuthStatusQuery
         */
        CmdAuthStatusQuery.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdAuthStatusQuery)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdAuthStatusQuery: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdAuthStatusQuery();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdAuthStatusQuery message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdAuthStatusQuery
         * @static
         * @param {usb_control.CmdAuthStatusQuery} message CmdAuthStatusQuery
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdAuthStatusQuery.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdAuthStatusQuery to JSON.
         * @function toJSON
         * @memberof usb_control.CmdAuthStatusQuery
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdAuthStatusQuery.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdAuthStatusQuery
         * @function getTypeUrl
         * @memberof usb_control.CmdAuthStatusQuery
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdAuthStatusQuery.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdAuthStatusQuery";
        };

        return CmdAuthStatusQuery;
    })();

    usb_control.RspAuthStatus = (function() {

        /**
         * Properties of a RspAuthStatus.
         * @memberof usb_control
         * @interface IRspAuthStatus
         * @property {boolean|null} [authorized] RspAuthStatus authorized
         * @property {number|Long|null} [expireTime] RspAuthStatus expireTime
         * @property {string|null} [deviceDescription] RspAuthStatus deviceDescription
         * @property {string|null} [authStatus] RspAuthStatus authStatus
         */

        /**
         * Constructs a new RspAuthStatus.
         * @memberof usb_control
         * @classdesc Represents a RspAuthStatus.
         * @implements IRspAuthStatus
         * @constructor
         * @param {usb_control.IRspAuthStatus=} [properties] Properties to set
         */
        function RspAuthStatus(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspAuthStatus authorized.
         * @member {boolean} authorized
         * @memberof usb_control.RspAuthStatus
         * @instance
         */
        RspAuthStatus.prototype.authorized = false;

        /**
         * RspAuthStatus expireTime.
         * @member {number|Long} expireTime
         * @memberof usb_control.RspAuthStatus
         * @instance
         */
        RspAuthStatus.prototype.expireTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * RspAuthStatus deviceDescription.
         * @member {string} deviceDescription
         * @memberof usb_control.RspAuthStatus
         * @instance
         */
        RspAuthStatus.prototype.deviceDescription = "";

        /**
         * RspAuthStatus authStatus.
         * @member {string} authStatus
         * @memberof usb_control.RspAuthStatus
         * @instance
         */
        RspAuthStatus.prototype.authStatus = "";

        /**
         * Encodes the specified RspAuthStatus message. Does not implicitly {@link usb_control.RspAuthStatus.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspAuthStatus
         * @static
         * @param {usb_control.IRspAuthStatus} message RspAuthStatus message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspAuthStatus.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.authorized != null && Object.hasOwnProperty.call(message, "authorized"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.authorized);
            if (message.expireTime != null && Object.hasOwnProperty.call(message, "expireTime"))
                writer.uint32(/* id 2, wireType 0 =*/16).int64(message.expireTime);
            if (message.deviceDescription != null && Object.hasOwnProperty.call(message, "deviceDescription"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.deviceDescription);
            if (message.authStatus != null && Object.hasOwnProperty.call(message, "authStatus"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.authStatus);
            return writer;
        };

        /**
         * Encodes the specified RspAuthStatus message, length delimited. Does not implicitly {@link usb_control.RspAuthStatus.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspAuthStatus
         * @static
         * @param {usb_control.IRspAuthStatus} message RspAuthStatus message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspAuthStatus.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspAuthStatus message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspAuthStatus
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspAuthStatus} RspAuthStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspAuthStatus.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspAuthStatus();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.authorized = reader.bool();
                        break;
                    }
                case 2: {
                        message.expireTime = reader.int64();
                        break;
                    }
                case 3: {
                        message.deviceDescription = reader.string();
                        break;
                    }
                case 4: {
                        message.authStatus = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspAuthStatus message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspAuthStatus
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspAuthStatus} RspAuthStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspAuthStatus.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspAuthStatus message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspAuthStatus
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspAuthStatus} RspAuthStatus
         */
        RspAuthStatus.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspAuthStatus)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspAuthStatus: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspAuthStatus();
            if (object.authorized != null)
                message.authorized = Boolean(object.authorized);
            if (object.expireTime != null)
                if ($util.Long)
                    message.expireTime = $util.Long.fromValue(object.expireTime, false);
                else if (typeof object.expireTime === "string")
                    message.expireTime = parseInt(object.expireTime, 10);
                else if (typeof object.expireTime === "number")
                    message.expireTime = object.expireTime;
                else if (typeof object.expireTime === "object")
                    message.expireTime = new $util.LongBits(object.expireTime.low >>> 0, object.expireTime.high >>> 0).toNumber();
            if (object.deviceDescription != null)
                message.deviceDescription = String(object.deviceDescription);
            if (object.authStatus != null)
                message.authStatus = String(object.authStatus);
            return message;
        };

        /**
         * Creates a plain object from a RspAuthStatus message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspAuthStatus
         * @static
         * @param {usb_control.RspAuthStatus} message RspAuthStatus
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspAuthStatus.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.authorized = false;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.expireTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.expireTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.deviceDescription = "";
                object.authStatus = "";
            }
            if (message.authorized != null && Object.hasOwnProperty.call(message, "authorized"))
                object.authorized = message.authorized;
            if (message.expireTime != null && Object.hasOwnProperty.call(message, "expireTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.expireTime = typeof message.expireTime === "number" ? BigInt(message.expireTime) : $util.Long.fromBits(message.expireTime.low >>> 0, message.expireTime.high >>> 0, false).toBigInt();
                else if (typeof message.expireTime === "number")
                    object.expireTime = options.longs === String ? String(message.expireTime) : message.expireTime;
                else
                    object.expireTime = options.longs === String ? $util.Long.prototype.toString.call(message.expireTime) : options.longs === Number ? new $util.LongBits(message.expireTime.low >>> 0, message.expireTime.high >>> 0).toNumber() : message.expireTime;
            if (message.deviceDescription != null && Object.hasOwnProperty.call(message, "deviceDescription"))
                object.deviceDescription = message.deviceDescription;
            if (message.authStatus != null && Object.hasOwnProperty.call(message, "authStatus"))
                object.authStatus = message.authStatus;
            return object;
        };

        /**
         * Converts this RspAuthStatus to JSON.
         * @function toJSON
         * @memberof usb_control.RspAuthStatus
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspAuthStatus.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspAuthStatus
         * @function getTypeUrl
         * @memberof usb_control.RspAuthStatus
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspAuthStatus.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspAuthStatus";
        };

        return RspAuthStatus;
    })();

    usb_control.CmdGetMachineCode = (function() {

        /**
         * Properties of a CmdGetMachineCode.
         * @memberof usb_control
         * @interface ICmdGetMachineCode
         * @property {string|null} [sessionToken] CmdGetMachineCode sessionToken
         */

        /**
         * Constructs a new CmdGetMachineCode.
         * @memberof usb_control
         * @classdesc Represents a CmdGetMachineCode.
         * @implements ICmdGetMachineCode
         * @constructor
         * @param {usb_control.ICmdGetMachineCode=} [properties] Properties to set
         */
        function CmdGetMachineCode(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdGetMachineCode sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdGetMachineCode
         * @instance
         */
        CmdGetMachineCode.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdGetMachineCode message. Does not implicitly {@link usb_control.CmdGetMachineCode.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdGetMachineCode
         * @static
         * @param {usb_control.ICmdGetMachineCode} message CmdGetMachineCode message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdGetMachineCode.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdGetMachineCode message, length delimited. Does not implicitly {@link usb_control.CmdGetMachineCode.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdGetMachineCode
         * @static
         * @param {usb_control.ICmdGetMachineCode} message CmdGetMachineCode message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdGetMachineCode.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdGetMachineCode message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdGetMachineCode
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdGetMachineCode} CmdGetMachineCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdGetMachineCode.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdGetMachineCode();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdGetMachineCode message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdGetMachineCode
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdGetMachineCode} CmdGetMachineCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdGetMachineCode.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdGetMachineCode message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdGetMachineCode
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdGetMachineCode} CmdGetMachineCode
         */
        CmdGetMachineCode.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdGetMachineCode)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdGetMachineCode: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdGetMachineCode();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdGetMachineCode message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdGetMachineCode
         * @static
         * @param {usb_control.CmdGetMachineCode} message CmdGetMachineCode
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdGetMachineCode.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdGetMachineCode to JSON.
         * @function toJSON
         * @memberof usb_control.CmdGetMachineCode
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdGetMachineCode.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdGetMachineCode
         * @function getTypeUrl
         * @memberof usb_control.CmdGetMachineCode
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdGetMachineCode.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdGetMachineCode";
        };

        return CmdGetMachineCode;
    })();

    usb_control.RspMachineCode = (function() {

        /**
         * Properties of a RspMachineCode.
         * @memberof usb_control
         * @interface IRspMachineCode
         * @property {string|null} [machineCode] RspMachineCode machineCode
         * @property {Uint8Array|null} [qrcodePng] RspMachineCode qrcodePng
         */

        /**
         * Constructs a new RspMachineCode.
         * @memberof usb_control
         * @classdesc Represents a RspMachineCode.
         * @implements IRspMachineCode
         * @constructor
         * @param {usb_control.IRspMachineCode=} [properties] Properties to set
         */
        function RspMachineCode(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspMachineCode machineCode.
         * @member {string} machineCode
         * @memberof usb_control.RspMachineCode
         * @instance
         */
        RspMachineCode.prototype.machineCode = "";

        /**
         * RspMachineCode qrcodePng.
         * @member {Uint8Array} qrcodePng
         * @memberof usb_control.RspMachineCode
         * @instance
         */
        RspMachineCode.prototype.qrcodePng = $util.newBuffer([]);

        /**
         * Encodes the specified RspMachineCode message. Does not implicitly {@link usb_control.RspMachineCode.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspMachineCode
         * @static
         * @param {usb_control.IRspMachineCode} message RspMachineCode message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspMachineCode.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.machineCode != null && Object.hasOwnProperty.call(message, "machineCode"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.machineCode);
            if (message.qrcodePng != null && Object.hasOwnProperty.call(message, "qrcodePng"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.qrcodePng);
            return writer;
        };

        /**
         * Encodes the specified RspMachineCode message, length delimited. Does not implicitly {@link usb_control.RspMachineCode.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspMachineCode
         * @static
         * @param {usb_control.IRspMachineCode} message RspMachineCode message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspMachineCode.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspMachineCode message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspMachineCode
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspMachineCode} RspMachineCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspMachineCode.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspMachineCode();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.machineCode = reader.string();
                        break;
                    }
                case 2: {
                        message.qrcodePng = reader.bytes();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspMachineCode message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspMachineCode
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspMachineCode} RspMachineCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspMachineCode.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspMachineCode message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspMachineCode
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspMachineCode} RspMachineCode
         */
        RspMachineCode.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspMachineCode)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspMachineCode: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspMachineCode();
            if (object.machineCode != null)
                message.machineCode = String(object.machineCode);
            if (object.qrcodePng != null)
                if (typeof object.qrcodePng === "string")
                    $util.base64.decode(object.qrcodePng, message.qrcodePng = $util.newBuffer($util.base64.length(object.qrcodePng)), 0);
                else if (object.qrcodePng.length >= 0)
                    message.qrcodePng = object.qrcodePng;
            return message;
        };

        /**
         * Creates a plain object from a RspMachineCode message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspMachineCode
         * @static
         * @param {usb_control.RspMachineCode} message RspMachineCode
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspMachineCode.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.machineCode = "";
                if (options.bytes === String)
                    object.qrcodePng = "";
                else {
                    object.qrcodePng = [];
                    if (options.bytes !== Array)
                        object.qrcodePng = $util.newBuffer(object.qrcodePng);
                }
            }
            if (message.machineCode != null && Object.hasOwnProperty.call(message, "machineCode"))
                object.machineCode = message.machineCode;
            if (message.qrcodePng != null && Object.hasOwnProperty.call(message, "qrcodePng"))
                object.qrcodePng = options.bytes === String ? $util.base64.encode(message.qrcodePng, 0, message.qrcodePng.length) : options.bytes === Array ? Array.prototype.slice.call(message.qrcodePng) : message.qrcodePng;
            return object;
        };

        /**
         * Converts this RspMachineCode to JSON.
         * @function toJSON
         * @memberof usb_control.RspMachineCode
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspMachineCode.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspMachineCode
         * @function getTypeUrl
         * @memberof usb_control.RspMachineCode
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspMachineCode.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspMachineCode";
        };

        return RspMachineCode;
    })();

    usb_control.CmdUploadLicense = (function() {

        /**
         * Properties of a CmdUploadLicense.
         * @memberof usb_control
         * @interface ICmdUploadLicense
         * @property {string|null} [sessionToken] CmdUploadLicense sessionToken
         * @property {Uint8Array|null} [licenseData] CmdUploadLicense licenseData
         */

        /**
         * Constructs a new CmdUploadLicense.
         * @memberof usb_control
         * @classdesc Represents a CmdUploadLicense.
         * @implements ICmdUploadLicense
         * @constructor
         * @param {usb_control.ICmdUploadLicense=} [properties] Properties to set
         */
        function CmdUploadLicense(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdUploadLicense sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdUploadLicense
         * @instance
         */
        CmdUploadLicense.prototype.sessionToken = "";

        /**
         * CmdUploadLicense licenseData.
         * @member {Uint8Array} licenseData
         * @memberof usb_control.CmdUploadLicense
         * @instance
         */
        CmdUploadLicense.prototype.licenseData = $util.newBuffer([]);

        /**
         * Encodes the specified CmdUploadLicense message. Does not implicitly {@link usb_control.CmdUploadLicense.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdUploadLicense
         * @static
         * @param {usb_control.ICmdUploadLicense} message CmdUploadLicense message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUploadLicense.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.licenseData != null && Object.hasOwnProperty.call(message, "licenseData"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.licenseData);
            return writer;
        };

        /**
         * Encodes the specified CmdUploadLicense message, length delimited. Does not implicitly {@link usb_control.CmdUploadLicense.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdUploadLicense
         * @static
         * @param {usb_control.ICmdUploadLicense} message CmdUploadLicense message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUploadLicense.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdUploadLicense message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdUploadLicense
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdUploadLicense} CmdUploadLicense
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUploadLicense.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdUploadLicense();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.licenseData = reader.bytes();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdUploadLicense message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdUploadLicense
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdUploadLicense} CmdUploadLicense
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUploadLicense.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdUploadLicense message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdUploadLicense
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdUploadLicense} CmdUploadLicense
         */
        CmdUploadLicense.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdUploadLicense)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdUploadLicense: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdUploadLicense();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.licenseData != null)
                if (typeof object.licenseData === "string")
                    $util.base64.decode(object.licenseData, message.licenseData = $util.newBuffer($util.base64.length(object.licenseData)), 0);
                else if (object.licenseData.length >= 0)
                    message.licenseData = object.licenseData;
            return message;
        };

        /**
         * Creates a plain object from a CmdUploadLicense message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdUploadLicense
         * @static
         * @param {usb_control.CmdUploadLicense} message CmdUploadLicense
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdUploadLicense.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                if (options.bytes === String)
                    object.licenseData = "";
                else {
                    object.licenseData = [];
                    if (options.bytes !== Array)
                        object.licenseData = $util.newBuffer(object.licenseData);
                }
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.licenseData != null && Object.hasOwnProperty.call(message, "licenseData"))
                object.licenseData = options.bytes === String ? $util.base64.encode(message.licenseData, 0, message.licenseData.length) : options.bytes === Array ? Array.prototype.slice.call(message.licenseData) : message.licenseData;
            return object;
        };

        /**
         * Converts this CmdUploadLicense to JSON.
         * @function toJSON
         * @memberof usb_control.CmdUploadLicense
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdUploadLicense.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdUploadLicense
         * @function getTypeUrl
         * @memberof usb_control.CmdUploadLicense
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdUploadLicense.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdUploadLicense";
        };

        return CmdUploadLicense;
    })();

    usb_control.RspUploadLicense = (function() {

        /**
         * Properties of a RspUploadLicense.
         * @memberof usb_control
         * @interface IRspUploadLicense
         * @property {boolean|null} [success] RspUploadLicense success
         * @property {number|Long|null} [expireTime] RspUploadLicense expireTime
         * @property {number|null} [resultCode] RspUploadLicense resultCode
         * @property {string|null} [errorMessage] RspUploadLicense errorMessage
         */

        /**
         * Constructs a new RspUploadLicense.
         * @memberof usb_control
         * @classdesc Represents a RspUploadLicense.
         * @implements IRspUploadLicense
         * @constructor
         * @param {usb_control.IRspUploadLicense=} [properties] Properties to set
         */
        function RspUploadLicense(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspUploadLicense success.
         * @member {boolean} success
         * @memberof usb_control.RspUploadLicense
         * @instance
         */
        RspUploadLicense.prototype.success = false;

        /**
         * RspUploadLicense expireTime.
         * @member {number|Long} expireTime
         * @memberof usb_control.RspUploadLicense
         * @instance
         */
        RspUploadLicense.prototype.expireTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * RspUploadLicense resultCode.
         * @member {number} resultCode
         * @memberof usb_control.RspUploadLicense
         * @instance
         */
        RspUploadLicense.prototype.resultCode = 0;

        /**
         * RspUploadLicense errorMessage.
         * @member {string} errorMessage
         * @memberof usb_control.RspUploadLicense
         * @instance
         */
        RspUploadLicense.prototype.errorMessage = "";

        /**
         * Encodes the specified RspUploadLicense message. Does not implicitly {@link usb_control.RspUploadLicense.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspUploadLicense
         * @static
         * @param {usb_control.IRspUploadLicense} message RspUploadLicense message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspUploadLicense.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
            if (message.expireTime != null && Object.hasOwnProperty.call(message, "expireTime"))
                writer.uint32(/* id 2, wireType 0 =*/16).int64(message.expireTime);
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                writer.uint32(/* id 3, wireType 0 =*/24).int32(message.resultCode);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.errorMessage);
            return writer;
        };

        /**
         * Encodes the specified RspUploadLicense message, length delimited. Does not implicitly {@link usb_control.RspUploadLicense.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspUploadLicense
         * @static
         * @param {usb_control.IRspUploadLicense} message RspUploadLicense message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspUploadLicense.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspUploadLicense message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspUploadLicense
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspUploadLicense} RspUploadLicense
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspUploadLicense.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspUploadLicense();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.success = reader.bool();
                        break;
                    }
                case 2: {
                        message.expireTime = reader.int64();
                        break;
                    }
                case 3: {
                        message.resultCode = reader.int32();
                        break;
                    }
                case 4: {
                        message.errorMessage = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspUploadLicense message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspUploadLicense
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspUploadLicense} RspUploadLicense
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspUploadLicense.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspUploadLicense message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspUploadLicense
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspUploadLicense} RspUploadLicense
         */
        RspUploadLicense.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspUploadLicense)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspUploadLicense: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspUploadLicense();
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.expireTime != null)
                if ($util.Long)
                    message.expireTime = $util.Long.fromValue(object.expireTime, false);
                else if (typeof object.expireTime === "string")
                    message.expireTime = parseInt(object.expireTime, 10);
                else if (typeof object.expireTime === "number")
                    message.expireTime = object.expireTime;
                else if (typeof object.expireTime === "object")
                    message.expireTime = new $util.LongBits(object.expireTime.low >>> 0, object.expireTime.high >>> 0).toNumber();
            if (object.resultCode != null)
                message.resultCode = object.resultCode | 0;
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            return message;
        };

        /**
         * Creates a plain object from a RspUploadLicense message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspUploadLicense
         * @static
         * @param {usb_control.RspUploadLicense} message RspUploadLicense
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspUploadLicense.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.success = false;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.expireTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.expireTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.resultCode = 0;
                object.errorMessage = "";
            }
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                object.success = message.success;
            if (message.expireTime != null && Object.hasOwnProperty.call(message, "expireTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.expireTime = typeof message.expireTime === "number" ? BigInt(message.expireTime) : $util.Long.fromBits(message.expireTime.low >>> 0, message.expireTime.high >>> 0, false).toBigInt();
                else if (typeof message.expireTime === "number")
                    object.expireTime = options.longs === String ? String(message.expireTime) : message.expireTime;
                else
                    object.expireTime = options.longs === String ? $util.Long.prototype.toString.call(message.expireTime) : options.longs === Number ? new $util.LongBits(message.expireTime.low >>> 0, message.expireTime.high >>> 0).toNumber() : message.expireTime;
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                object.resultCode = message.resultCode;
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                object.errorMessage = message.errorMessage;
            return object;
        };

        /**
         * Converts this RspUploadLicense to JSON.
         * @function toJSON
         * @memberof usb_control.RspUploadLicense
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspUploadLicense.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspUploadLicense
         * @function getTypeUrl
         * @memberof usb_control.RspUploadLicense
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspUploadLicense.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspUploadLicense";
        };

        return RspUploadLicense;
    })();

    usb_control.CmdLogout = (function() {

        /**
         * Properties of a CmdLogout.
         * @memberof usb_control
         * @interface ICmdLogout
         * @property {string|null} [sessionToken] CmdLogout sessionToken
         */

        /**
         * Constructs a new CmdLogout.
         * @memberof usb_control
         * @classdesc Represents a CmdLogout.
         * @implements ICmdLogout
         * @constructor
         * @param {usb_control.ICmdLogout=} [properties] Properties to set
         */
        function CmdLogout(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdLogout sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdLogout
         * @instance
         */
        CmdLogout.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdLogout message. Does not implicitly {@link usb_control.CmdLogout.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdLogout
         * @static
         * @param {usb_control.ICmdLogout} message CmdLogout message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdLogout.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdLogout message, length delimited. Does not implicitly {@link usb_control.CmdLogout.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdLogout
         * @static
         * @param {usb_control.ICmdLogout} message CmdLogout message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdLogout.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdLogout message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdLogout
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdLogout} CmdLogout
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdLogout.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdLogout();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdLogout message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdLogout
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdLogout} CmdLogout
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdLogout.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdLogout message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdLogout
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdLogout} CmdLogout
         */
        CmdLogout.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdLogout)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdLogout: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdLogout();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdLogout message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdLogout
         * @static
         * @param {usb_control.CmdLogout} message CmdLogout
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdLogout.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdLogout to JSON.
         * @function toJSON
         * @memberof usb_control.CmdLogout
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdLogout.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdLogout
         * @function getTypeUrl
         * @memberof usb_control.CmdLogout
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdLogout.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdLogout";
        };

        return CmdLogout;
    })();

    usb_control.WhitelistDevice = (function() {

        /**
         * Properties of a WhitelistDevice.
         * @memberof usb_control
         * @interface IWhitelistDevice
         * @property {string|null} [serialNumber] WhitelistDevice serialNumber
         * @property {string|null} [vid] WhitelistDevice vid
         * @property {string|null} [pid] WhitelistDevice pid
         * @property {string|null} [deviceName] WhitelistDevice deviceName
         * @property {number|Long|null} [capacityBytes] WhitelistDevice capacityBytes
         * @property {string|null} [permission] WhitelistDevice permission
         * @property {string|null} [description] WhitelistDevice description
         * @property {string|null} [addMethod] WhitelistDevice addMethod
         * @property {number|Long|null} [createdAt] WhitelistDevice createdAt
         * @property {string|null} [deviceType] WhitelistDevice deviceType
         */

        /**
         * Constructs a new WhitelistDevice.
         * @memberof usb_control
         * @classdesc Represents a WhitelistDevice.
         * @implements IWhitelistDevice
         * @constructor
         * @param {usb_control.IWhitelistDevice=} [properties] Properties to set
         */
        function WhitelistDevice(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * WhitelistDevice serialNumber.
         * @member {string} serialNumber
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.serialNumber = "";

        /**
         * WhitelistDevice vid.
         * @member {string} vid
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.vid = "";

        /**
         * WhitelistDevice pid.
         * @member {string} pid
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.pid = "";

        /**
         * WhitelistDevice deviceName.
         * @member {string} deviceName
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.deviceName = "";

        /**
         * WhitelistDevice capacityBytes.
         * @member {number|Long} capacityBytes
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.capacityBytes = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * WhitelistDevice permission.
         * @member {string} permission
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.permission = "";

        /**
         * WhitelistDevice description.
         * @member {string} description
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.description = "";

        /**
         * WhitelistDevice addMethod.
         * @member {string} addMethod
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.addMethod = "";

        /**
         * WhitelistDevice createdAt.
         * @member {number|Long} createdAt
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.createdAt = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * WhitelistDevice deviceType.
         * @member {string} deviceType
         * @memberof usb_control.WhitelistDevice
         * @instance
         */
        WhitelistDevice.prototype.deviceType = "";

        /**
         * Encodes the specified WhitelistDevice message. Does not implicitly {@link usb_control.WhitelistDevice.verify|verify} messages.
         * @function encode
         * @memberof usb_control.WhitelistDevice
         * @static
         * @param {usb_control.IWhitelistDevice} message WhitelistDevice message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        WhitelistDevice.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.serialNumber);
            if (message.vid != null && Object.hasOwnProperty.call(message, "vid"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.vid);
            if (message.pid != null && Object.hasOwnProperty.call(message, "pid"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.pid);
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.deviceName);
            if (message.capacityBytes != null && Object.hasOwnProperty.call(message, "capacityBytes"))
                writer.uint32(/* id 5, wireType 0 =*/40).int64(message.capacityBytes);
            if (message.permission != null && Object.hasOwnProperty.call(message, "permission"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.permission);
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.description);
            if (message.addMethod != null && Object.hasOwnProperty.call(message, "addMethod"))
                writer.uint32(/* id 8, wireType 2 =*/66).string(message.addMethod);
            if (message.createdAt != null && Object.hasOwnProperty.call(message, "createdAt"))
                writer.uint32(/* id 9, wireType 0 =*/72).int64(message.createdAt);
            if (message.deviceType != null && Object.hasOwnProperty.call(message, "deviceType"))
                writer.uint32(/* id 10, wireType 2 =*/82).string(message.deviceType);
            return writer;
        };

        /**
         * Encodes the specified WhitelistDevice message, length delimited. Does not implicitly {@link usb_control.WhitelistDevice.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.WhitelistDevice
         * @static
         * @param {usb_control.IWhitelistDevice} message WhitelistDevice message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        WhitelistDevice.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a WhitelistDevice message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.WhitelistDevice
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.WhitelistDevice} WhitelistDevice
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        WhitelistDevice.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.WhitelistDevice();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.serialNumber = reader.string();
                        break;
                    }
                case 2: {
                        message.vid = reader.string();
                        break;
                    }
                case 3: {
                        message.pid = reader.string();
                        break;
                    }
                case 4: {
                        message.deviceName = reader.string();
                        break;
                    }
                case 5: {
                        message.capacityBytes = reader.int64();
                        break;
                    }
                case 6: {
                        message.permission = reader.string();
                        break;
                    }
                case 7: {
                        message.description = reader.string();
                        break;
                    }
                case 8: {
                        message.addMethod = reader.string();
                        break;
                    }
                case 9: {
                        message.createdAt = reader.int64();
                        break;
                    }
                case 10: {
                        message.deviceType = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a WhitelistDevice message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.WhitelistDevice
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.WhitelistDevice} WhitelistDevice
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        WhitelistDevice.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a WhitelistDevice message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.WhitelistDevice
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.WhitelistDevice} WhitelistDevice
         */
        WhitelistDevice.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.WhitelistDevice)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.WhitelistDevice: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.WhitelistDevice();
            if (object.serialNumber != null)
                message.serialNumber = String(object.serialNumber);
            if (object.vid != null)
                message.vid = String(object.vid);
            if (object.pid != null)
                message.pid = String(object.pid);
            if (object.deviceName != null)
                message.deviceName = String(object.deviceName);
            if (object.capacityBytes != null)
                if ($util.Long)
                    message.capacityBytes = $util.Long.fromValue(object.capacityBytes, false);
                else if (typeof object.capacityBytes === "string")
                    message.capacityBytes = parseInt(object.capacityBytes, 10);
                else if (typeof object.capacityBytes === "number")
                    message.capacityBytes = object.capacityBytes;
                else if (typeof object.capacityBytes === "object")
                    message.capacityBytes = new $util.LongBits(object.capacityBytes.low >>> 0, object.capacityBytes.high >>> 0).toNumber();
            if (object.permission != null)
                message.permission = String(object.permission);
            if (object.description != null)
                message.description = String(object.description);
            if (object.addMethod != null)
                message.addMethod = String(object.addMethod);
            if (object.createdAt != null)
                if ($util.Long)
                    message.createdAt = $util.Long.fromValue(object.createdAt, false);
                else if (typeof object.createdAt === "string")
                    message.createdAt = parseInt(object.createdAt, 10);
                else if (typeof object.createdAt === "number")
                    message.createdAt = object.createdAt;
                else if (typeof object.createdAt === "object")
                    message.createdAt = new $util.LongBits(object.createdAt.low >>> 0, object.createdAt.high >>> 0).toNumber();
            if (object.deviceType != null)
                message.deviceType = String(object.deviceType);
            return message;
        };

        /**
         * Creates a plain object from a WhitelistDevice message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.WhitelistDevice
         * @static
         * @param {usb_control.WhitelistDevice} message WhitelistDevice
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        WhitelistDevice.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.serialNumber = "";
                object.vid = "";
                object.pid = "";
                object.deviceName = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.capacityBytes = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.capacityBytes = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.permission = "";
                object.description = "";
                object.addMethod = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.createdAt = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.createdAt = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.deviceType = "";
            }
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                object.serialNumber = message.serialNumber;
            if (message.vid != null && Object.hasOwnProperty.call(message, "vid"))
                object.vid = message.vid;
            if (message.pid != null && Object.hasOwnProperty.call(message, "pid"))
                object.pid = message.pid;
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                object.deviceName = message.deviceName;
            if (message.capacityBytes != null && Object.hasOwnProperty.call(message, "capacityBytes"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.capacityBytes = typeof message.capacityBytes === "number" ? BigInt(message.capacityBytes) : $util.Long.fromBits(message.capacityBytes.low >>> 0, message.capacityBytes.high >>> 0, false).toBigInt();
                else if (typeof message.capacityBytes === "number")
                    object.capacityBytes = options.longs === String ? String(message.capacityBytes) : message.capacityBytes;
                else
                    object.capacityBytes = options.longs === String ? $util.Long.prototype.toString.call(message.capacityBytes) : options.longs === Number ? new $util.LongBits(message.capacityBytes.low >>> 0, message.capacityBytes.high >>> 0).toNumber() : message.capacityBytes;
            if (message.permission != null && Object.hasOwnProperty.call(message, "permission"))
                object.permission = message.permission;
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                object.description = message.description;
            if (message.addMethod != null && Object.hasOwnProperty.call(message, "addMethod"))
                object.addMethod = message.addMethod;
            if (message.createdAt != null && Object.hasOwnProperty.call(message, "createdAt"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.createdAt = typeof message.createdAt === "number" ? BigInt(message.createdAt) : $util.Long.fromBits(message.createdAt.low >>> 0, message.createdAt.high >>> 0, false).toBigInt();
                else if (typeof message.createdAt === "number")
                    object.createdAt = options.longs === String ? String(message.createdAt) : message.createdAt;
                else
                    object.createdAt = options.longs === String ? $util.Long.prototype.toString.call(message.createdAt) : options.longs === Number ? new $util.LongBits(message.createdAt.low >>> 0, message.createdAt.high >>> 0).toNumber() : message.createdAt;
            if (message.deviceType != null && Object.hasOwnProperty.call(message, "deviceType"))
                object.deviceType = message.deviceType;
            return object;
        };

        /**
         * Converts this WhitelistDevice to JSON.
         * @function toJSON
         * @memberof usb_control.WhitelistDevice
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        WhitelistDevice.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for WhitelistDevice
         * @function getTypeUrl
         * @memberof usb_control.WhitelistDevice
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        WhitelistDevice.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.WhitelistDevice";
        };

        return WhitelistDevice;
    })();

    usb_control.CmdListWhitelist = (function() {

        /**
         * Properties of a CmdListWhitelist.
         * @memberof usb_control
         * @interface ICmdListWhitelist
         * @property {string|null} [sessionToken] CmdListWhitelist sessionToken
         */

        /**
         * Constructs a new CmdListWhitelist.
         * @memberof usb_control
         * @classdesc Represents a CmdListWhitelist.
         * @implements ICmdListWhitelist
         * @constructor
         * @param {usb_control.ICmdListWhitelist=} [properties] Properties to set
         */
        function CmdListWhitelist(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdListWhitelist sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdListWhitelist
         * @instance
         */
        CmdListWhitelist.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdListWhitelist message. Does not implicitly {@link usb_control.CmdListWhitelist.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdListWhitelist
         * @static
         * @param {usb_control.ICmdListWhitelist} message CmdListWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdListWhitelist.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdListWhitelist message, length delimited. Does not implicitly {@link usb_control.CmdListWhitelist.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdListWhitelist
         * @static
         * @param {usb_control.ICmdListWhitelist} message CmdListWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdListWhitelist.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdListWhitelist message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdListWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdListWhitelist} CmdListWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdListWhitelist.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdListWhitelist();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdListWhitelist message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdListWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdListWhitelist} CmdListWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdListWhitelist.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdListWhitelist message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdListWhitelist
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdListWhitelist} CmdListWhitelist
         */
        CmdListWhitelist.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdListWhitelist)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdListWhitelist: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdListWhitelist();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdListWhitelist message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdListWhitelist
         * @static
         * @param {usb_control.CmdListWhitelist} message CmdListWhitelist
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdListWhitelist.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdListWhitelist to JSON.
         * @function toJSON
         * @memberof usb_control.CmdListWhitelist
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdListWhitelist.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdListWhitelist
         * @function getTypeUrl
         * @memberof usb_control.CmdListWhitelist
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdListWhitelist.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdListWhitelist";
        };

        return CmdListWhitelist;
    })();

    usb_control.RspListWhitelist = (function() {

        /**
         * Properties of a RspListWhitelist.
         * @memberof usb_control
         * @interface IRspListWhitelist
         * @property {Array.<usb_control.IWhitelistDevice>|null} [devices] RspListWhitelist devices
         */

        /**
         * Constructs a new RspListWhitelist.
         * @memberof usb_control
         * @classdesc Represents a RspListWhitelist.
         * @implements IRspListWhitelist
         * @constructor
         * @param {usb_control.IRspListWhitelist=} [properties] Properties to set
         */
        function RspListWhitelist(properties) {
            this.devices = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspListWhitelist devices.
         * @member {Array.<usb_control.IWhitelistDevice>} devices
         * @memberof usb_control.RspListWhitelist
         * @instance
         */
        RspListWhitelist.prototype.devices = $util.emptyArray;

        /**
         * Encodes the specified RspListWhitelist message. Does not implicitly {@link usb_control.RspListWhitelist.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspListWhitelist
         * @static
         * @param {usb_control.IRspListWhitelist} message RspListWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspListWhitelist.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.devices != null && message.devices.length)
                for (let i = 0; i < message.devices.length; ++i)
                    $root.usb_control.WhitelistDevice.encode(message.devices[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
            return writer;
        };

        /**
         * Encodes the specified RspListWhitelist message, length delimited. Does not implicitly {@link usb_control.RspListWhitelist.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspListWhitelist
         * @static
         * @param {usb_control.IRspListWhitelist} message RspListWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspListWhitelist.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspListWhitelist message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspListWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspListWhitelist} RspListWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspListWhitelist.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspListWhitelist();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        if (!(message.devices && message.devices.length))
                            message.devices = [];
                        message.devices.push($root.usb_control.WhitelistDevice.decode(reader, reader.uint32(), undefined, long + 1));
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspListWhitelist message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspListWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspListWhitelist} RspListWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspListWhitelist.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspListWhitelist message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspListWhitelist
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspListWhitelist} RspListWhitelist
         */
        RspListWhitelist.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspListWhitelist)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspListWhitelist: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspListWhitelist();
            if (object.devices) {
                if (!Array.isArray(object.devices))
                    throw TypeError(".usb_control.RspListWhitelist.devices: array expected");
                message.devices = [];
                for (let i = 0; i < object.devices.length; ++i) {
                    if (!$util.isObject(object.devices[i]))
                        throw TypeError(".usb_control.RspListWhitelist.devices: object expected");
                    message.devices[i] = $root.usb_control.WhitelistDevice.fromObject(object.devices[i], long + 1);
                }
            }
            return message;
        };

        /**
         * Creates a plain object from a RspListWhitelist message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspListWhitelist
         * @static
         * @param {usb_control.RspListWhitelist} message RspListWhitelist
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspListWhitelist.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.arrays || options.defaults)
                object.devices = [];
            if (message.devices && message.devices.length) {
                object.devices = [];
                for (let j = 0; j < message.devices.length; ++j)
                    object.devices[j] = $root.usb_control.WhitelistDevice.toObject(message.devices[j], options, q + 1);
            }
            return object;
        };

        /**
         * Converts this RspListWhitelist to JSON.
         * @function toJSON
         * @memberof usb_control.RspListWhitelist
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspListWhitelist.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspListWhitelist
         * @function getTypeUrl
         * @memberof usb_control.RspListWhitelist
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspListWhitelist.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspListWhitelist";
        };

        return RspListWhitelist;
    })();

    usb_control.ConnectedDevice = (function() {

        /**
         * Properties of a ConnectedDevice.
         * @memberof usb_control
         * @interface IConnectedDevice
         * @property {string|null} [serialNumber] ConnectedDevice serialNumber
         * @property {string|null} [deviceName] ConnectedDevice deviceName
         * @property {string|null} [vid] ConnectedDevice vid
         * @property {string|null} [pid] ConnectedDevice pid
         * @property {number|Long|null} [capacityBytes] ConnectedDevice capacityBytes
         * @property {string|null} [deviceType] ConnectedDevice deviceType
         * @property {string|null} [interfaceType] ConnectedDevice interfaceType
         * @property {string|null} [admissionStatus] ConnectedDevice admissionStatus
         * @property {string|null} [failReason] ConnectedDevice failReason
         */

        /**
         * Constructs a new ConnectedDevice.
         * @memberof usb_control
         * @classdesc Represents a ConnectedDevice.
         * @implements IConnectedDevice
         * @constructor
         * @param {usb_control.IConnectedDevice=} [properties] Properties to set
         */
        function ConnectedDevice(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ConnectedDevice serialNumber.
         * @member {string} serialNumber
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.serialNumber = "";

        /**
         * ConnectedDevice deviceName.
         * @member {string} deviceName
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.deviceName = "";

        /**
         * ConnectedDevice vid.
         * @member {string} vid
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.vid = "";

        /**
         * ConnectedDevice pid.
         * @member {string} pid
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.pid = "";

        /**
         * ConnectedDevice capacityBytes.
         * @member {number|Long} capacityBytes
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.capacityBytes = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * ConnectedDevice deviceType.
         * @member {string} deviceType
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.deviceType = "";

        /**
         * ConnectedDevice interfaceType.
         * @member {string} interfaceType
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.interfaceType = "";

        /**
         * ConnectedDevice admissionStatus.
         * @member {string} admissionStatus
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.admissionStatus = "";

        /**
         * ConnectedDevice failReason.
         * @member {string} failReason
         * @memberof usb_control.ConnectedDevice
         * @instance
         */
        ConnectedDevice.prototype.failReason = "";

        /**
         * Encodes the specified ConnectedDevice message. Does not implicitly {@link usb_control.ConnectedDevice.verify|verify} messages.
         * @function encode
         * @memberof usb_control.ConnectedDevice
         * @static
         * @param {usb_control.IConnectedDevice} message ConnectedDevice message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ConnectedDevice.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.serialNumber);
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.deviceName);
            if (message.vid != null && Object.hasOwnProperty.call(message, "vid"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.vid);
            if (message.pid != null && Object.hasOwnProperty.call(message, "pid"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.pid);
            if (message.capacityBytes != null && Object.hasOwnProperty.call(message, "capacityBytes"))
                writer.uint32(/* id 5, wireType 0 =*/40).int64(message.capacityBytes);
            if (message.deviceType != null && Object.hasOwnProperty.call(message, "deviceType"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.deviceType);
            if (message.interfaceType != null && Object.hasOwnProperty.call(message, "interfaceType"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.interfaceType);
            if (message.admissionStatus != null && Object.hasOwnProperty.call(message, "admissionStatus"))
                writer.uint32(/* id 8, wireType 2 =*/66).string(message.admissionStatus);
            if (message.failReason != null && Object.hasOwnProperty.call(message, "failReason"))
                writer.uint32(/* id 9, wireType 2 =*/74).string(message.failReason);
            return writer;
        };

        /**
         * Encodes the specified ConnectedDevice message, length delimited. Does not implicitly {@link usb_control.ConnectedDevice.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.ConnectedDevice
         * @static
         * @param {usb_control.IConnectedDevice} message ConnectedDevice message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ConnectedDevice.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a ConnectedDevice message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.ConnectedDevice
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.ConnectedDevice} ConnectedDevice
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ConnectedDevice.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.ConnectedDevice();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.serialNumber = reader.string();
                        break;
                    }
                case 2: {
                        message.deviceName = reader.string();
                        break;
                    }
                case 3: {
                        message.vid = reader.string();
                        break;
                    }
                case 4: {
                        message.pid = reader.string();
                        break;
                    }
                case 5: {
                        message.capacityBytes = reader.int64();
                        break;
                    }
                case 6: {
                        message.deviceType = reader.string();
                        break;
                    }
                case 7: {
                        message.interfaceType = reader.string();
                        break;
                    }
                case 8: {
                        message.admissionStatus = reader.string();
                        break;
                    }
                case 9: {
                        message.failReason = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ConnectedDevice message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.ConnectedDevice
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.ConnectedDevice} ConnectedDevice
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ConnectedDevice.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a ConnectedDevice message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.ConnectedDevice
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.ConnectedDevice} ConnectedDevice
         */
        ConnectedDevice.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.ConnectedDevice)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.ConnectedDevice: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.ConnectedDevice();
            if (object.serialNumber != null)
                message.serialNumber = String(object.serialNumber);
            if (object.deviceName != null)
                message.deviceName = String(object.deviceName);
            if (object.vid != null)
                message.vid = String(object.vid);
            if (object.pid != null)
                message.pid = String(object.pid);
            if (object.capacityBytes != null)
                if ($util.Long)
                    message.capacityBytes = $util.Long.fromValue(object.capacityBytes, false);
                else if (typeof object.capacityBytes === "string")
                    message.capacityBytes = parseInt(object.capacityBytes, 10);
                else if (typeof object.capacityBytes === "number")
                    message.capacityBytes = object.capacityBytes;
                else if (typeof object.capacityBytes === "object")
                    message.capacityBytes = new $util.LongBits(object.capacityBytes.low >>> 0, object.capacityBytes.high >>> 0).toNumber();
            if (object.deviceType != null)
                message.deviceType = String(object.deviceType);
            if (object.interfaceType != null)
                message.interfaceType = String(object.interfaceType);
            if (object.admissionStatus != null)
                message.admissionStatus = String(object.admissionStatus);
            if (object.failReason != null)
                message.failReason = String(object.failReason);
            return message;
        };

        /**
         * Creates a plain object from a ConnectedDevice message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.ConnectedDevice
         * @static
         * @param {usb_control.ConnectedDevice} message ConnectedDevice
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ConnectedDevice.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.serialNumber = "";
                object.deviceName = "";
                object.vid = "";
                object.pid = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.capacityBytes = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.capacityBytes = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.deviceType = "";
                object.interfaceType = "";
                object.admissionStatus = "";
                object.failReason = "";
            }
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                object.serialNumber = message.serialNumber;
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                object.deviceName = message.deviceName;
            if (message.vid != null && Object.hasOwnProperty.call(message, "vid"))
                object.vid = message.vid;
            if (message.pid != null && Object.hasOwnProperty.call(message, "pid"))
                object.pid = message.pid;
            if (message.capacityBytes != null && Object.hasOwnProperty.call(message, "capacityBytes"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.capacityBytes = typeof message.capacityBytes === "number" ? BigInt(message.capacityBytes) : $util.Long.fromBits(message.capacityBytes.low >>> 0, message.capacityBytes.high >>> 0, false).toBigInt();
                else if (typeof message.capacityBytes === "number")
                    object.capacityBytes = options.longs === String ? String(message.capacityBytes) : message.capacityBytes;
                else
                    object.capacityBytes = options.longs === String ? $util.Long.prototype.toString.call(message.capacityBytes) : options.longs === Number ? new $util.LongBits(message.capacityBytes.low >>> 0, message.capacityBytes.high >>> 0).toNumber() : message.capacityBytes;
            if (message.deviceType != null && Object.hasOwnProperty.call(message, "deviceType"))
                object.deviceType = message.deviceType;
            if (message.interfaceType != null && Object.hasOwnProperty.call(message, "interfaceType"))
                object.interfaceType = message.interfaceType;
            if (message.admissionStatus != null && Object.hasOwnProperty.call(message, "admissionStatus"))
                object.admissionStatus = message.admissionStatus;
            if (message.failReason != null && Object.hasOwnProperty.call(message, "failReason"))
                object.failReason = message.failReason;
            return object;
        };

        /**
         * Converts this ConnectedDevice to JSON.
         * @function toJSON
         * @memberof usb_control.ConnectedDevice
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ConnectedDevice.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ConnectedDevice
         * @function getTypeUrl
         * @memberof usb_control.ConnectedDevice
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ConnectedDevice.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.ConnectedDevice";
        };

        return ConnectedDevice;
    })();

    usb_control.CmdGetConnectedDevices = (function() {

        /**
         * Properties of a CmdGetConnectedDevices.
         * @memberof usb_control
         * @interface ICmdGetConnectedDevices
         * @property {string|null} [sessionToken] CmdGetConnectedDevices sessionToken
         */

        /**
         * Constructs a new CmdGetConnectedDevices.
         * @memberof usb_control
         * @classdesc Represents a CmdGetConnectedDevices.
         * @implements ICmdGetConnectedDevices
         * @constructor
         * @param {usb_control.ICmdGetConnectedDevices=} [properties] Properties to set
         */
        function CmdGetConnectedDevices(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdGetConnectedDevices sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdGetConnectedDevices
         * @instance
         */
        CmdGetConnectedDevices.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdGetConnectedDevices message. Does not implicitly {@link usb_control.CmdGetConnectedDevices.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdGetConnectedDevices
         * @static
         * @param {usb_control.ICmdGetConnectedDevices} message CmdGetConnectedDevices message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdGetConnectedDevices.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdGetConnectedDevices message, length delimited. Does not implicitly {@link usb_control.CmdGetConnectedDevices.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdGetConnectedDevices
         * @static
         * @param {usb_control.ICmdGetConnectedDevices} message CmdGetConnectedDevices message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdGetConnectedDevices.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdGetConnectedDevices message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdGetConnectedDevices
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdGetConnectedDevices} CmdGetConnectedDevices
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdGetConnectedDevices.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdGetConnectedDevices();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdGetConnectedDevices message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdGetConnectedDevices
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdGetConnectedDevices} CmdGetConnectedDevices
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdGetConnectedDevices.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdGetConnectedDevices message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdGetConnectedDevices
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdGetConnectedDevices} CmdGetConnectedDevices
         */
        CmdGetConnectedDevices.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdGetConnectedDevices)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdGetConnectedDevices: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdGetConnectedDevices();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdGetConnectedDevices message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdGetConnectedDevices
         * @static
         * @param {usb_control.CmdGetConnectedDevices} message CmdGetConnectedDevices
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdGetConnectedDevices.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdGetConnectedDevices to JSON.
         * @function toJSON
         * @memberof usb_control.CmdGetConnectedDevices
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdGetConnectedDevices.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdGetConnectedDevices
         * @function getTypeUrl
         * @memberof usb_control.CmdGetConnectedDevices
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdGetConnectedDevices.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdGetConnectedDevices";
        };

        return CmdGetConnectedDevices;
    })();

    usb_control.RspConnectedDevices = (function() {

        /**
         * Properties of a RspConnectedDevices.
         * @memberof usb_control
         * @interface IRspConnectedDevices
         * @property {Array.<usb_control.IConnectedDevice>|null} [devices] RspConnectedDevices devices
         */

        /**
         * Constructs a new RspConnectedDevices.
         * @memberof usb_control
         * @classdesc Represents a RspConnectedDevices.
         * @implements IRspConnectedDevices
         * @constructor
         * @param {usb_control.IRspConnectedDevices=} [properties] Properties to set
         */
        function RspConnectedDevices(properties) {
            this.devices = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspConnectedDevices devices.
         * @member {Array.<usb_control.IConnectedDevice>} devices
         * @memberof usb_control.RspConnectedDevices
         * @instance
         */
        RspConnectedDevices.prototype.devices = $util.emptyArray;

        /**
         * Encodes the specified RspConnectedDevices message. Does not implicitly {@link usb_control.RspConnectedDevices.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspConnectedDevices
         * @static
         * @param {usb_control.IRspConnectedDevices} message RspConnectedDevices message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspConnectedDevices.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.devices != null && message.devices.length)
                for (let i = 0; i < message.devices.length; ++i)
                    $root.usb_control.ConnectedDevice.encode(message.devices[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
            return writer;
        };

        /**
         * Encodes the specified RspConnectedDevices message, length delimited. Does not implicitly {@link usb_control.RspConnectedDevices.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspConnectedDevices
         * @static
         * @param {usb_control.IRspConnectedDevices} message RspConnectedDevices message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspConnectedDevices.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspConnectedDevices message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspConnectedDevices
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspConnectedDevices} RspConnectedDevices
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspConnectedDevices.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspConnectedDevices();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        if (!(message.devices && message.devices.length))
                            message.devices = [];
                        message.devices.push($root.usb_control.ConnectedDevice.decode(reader, reader.uint32(), undefined, long + 1));
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspConnectedDevices message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspConnectedDevices
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspConnectedDevices} RspConnectedDevices
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspConnectedDevices.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspConnectedDevices message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspConnectedDevices
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspConnectedDevices} RspConnectedDevices
         */
        RspConnectedDevices.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspConnectedDevices)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspConnectedDevices: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspConnectedDevices();
            if (object.devices) {
                if (!Array.isArray(object.devices))
                    throw TypeError(".usb_control.RspConnectedDevices.devices: array expected");
                message.devices = [];
                for (let i = 0; i < object.devices.length; ++i) {
                    if (!$util.isObject(object.devices[i]))
                        throw TypeError(".usb_control.RspConnectedDevices.devices: object expected");
                    message.devices[i] = $root.usb_control.ConnectedDevice.fromObject(object.devices[i], long + 1);
                }
            }
            return message;
        };

        /**
         * Creates a plain object from a RspConnectedDevices message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspConnectedDevices
         * @static
         * @param {usb_control.RspConnectedDevices} message RspConnectedDevices
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspConnectedDevices.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.arrays || options.defaults)
                object.devices = [];
            if (message.devices && message.devices.length) {
                object.devices = [];
                for (let j = 0; j < message.devices.length; ++j)
                    object.devices[j] = $root.usb_control.ConnectedDevice.toObject(message.devices[j], options, q + 1);
            }
            return object;
        };

        /**
         * Converts this RspConnectedDevices to JSON.
         * @function toJSON
         * @memberof usb_control.RspConnectedDevices
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspConnectedDevices.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspConnectedDevices
         * @function getTypeUrl
         * @memberof usb_control.RspConnectedDevices
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspConnectedDevices.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspConnectedDevices";
        };

        return RspConnectedDevices;
    })();

    usb_control.CmdAddWhitelist = (function() {

        /**
         * Properties of a CmdAddWhitelist.
         * @memberof usb_control
         * @interface ICmdAddWhitelist
         * @property {string|null} [sessionToken] CmdAddWhitelist sessionToken
         * @property {string|null} [serialNumber] CmdAddWhitelist serialNumber
         * @property {string|null} [vid] CmdAddWhitelist vid
         * @property {string|null} [pid] CmdAddWhitelist pid
         * @property {string|null} [deviceName] CmdAddWhitelist deviceName
         * @property {number|Long|null} [capacityBytes] CmdAddWhitelist capacityBytes
         * @property {string|null} [permission] CmdAddWhitelist permission
         * @property {string|null} [description] CmdAddWhitelist description
         * @property {string|null} [addMethod] CmdAddWhitelist addMethod
         * @property {string|null} [deviceType] CmdAddWhitelist deviceType
         */

        /**
         * Constructs a new CmdAddWhitelist.
         * @memberof usb_control
         * @classdesc Represents a CmdAddWhitelist.
         * @implements ICmdAddWhitelist
         * @constructor
         * @param {usb_control.ICmdAddWhitelist=} [properties] Properties to set
         */
        function CmdAddWhitelist(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdAddWhitelist sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.sessionToken = "";

        /**
         * CmdAddWhitelist serialNumber.
         * @member {string} serialNumber
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.serialNumber = "";

        /**
         * CmdAddWhitelist vid.
         * @member {string} vid
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.vid = "";

        /**
         * CmdAddWhitelist pid.
         * @member {string} pid
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.pid = "";

        /**
         * CmdAddWhitelist deviceName.
         * @member {string} deviceName
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.deviceName = "";

        /**
         * CmdAddWhitelist capacityBytes.
         * @member {number|Long} capacityBytes
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.capacityBytes = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * CmdAddWhitelist permission.
         * @member {string} permission
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.permission = "";

        /**
         * CmdAddWhitelist description.
         * @member {string} description
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.description = "";

        /**
         * CmdAddWhitelist addMethod.
         * @member {string} addMethod
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.addMethod = "";

        /**
         * CmdAddWhitelist deviceType.
         * @member {string} deviceType
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         */
        CmdAddWhitelist.prototype.deviceType = "";

        /**
         * Encodes the specified CmdAddWhitelist message. Does not implicitly {@link usb_control.CmdAddWhitelist.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdAddWhitelist
         * @static
         * @param {usb_control.ICmdAddWhitelist} message CmdAddWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdAddWhitelist.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.serialNumber);
            if (message.vid != null && Object.hasOwnProperty.call(message, "vid"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.vid);
            if (message.pid != null && Object.hasOwnProperty.call(message, "pid"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.pid);
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.deviceName);
            if (message.capacityBytes != null && Object.hasOwnProperty.call(message, "capacityBytes"))
                writer.uint32(/* id 6, wireType 0 =*/48).int64(message.capacityBytes);
            if (message.permission != null && Object.hasOwnProperty.call(message, "permission"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.permission);
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                writer.uint32(/* id 8, wireType 2 =*/66).string(message.description);
            if (message.addMethod != null && Object.hasOwnProperty.call(message, "addMethod"))
                writer.uint32(/* id 9, wireType 2 =*/74).string(message.addMethod);
            if (message.deviceType != null && Object.hasOwnProperty.call(message, "deviceType"))
                writer.uint32(/* id 10, wireType 2 =*/82).string(message.deviceType);
            return writer;
        };

        /**
         * Encodes the specified CmdAddWhitelist message, length delimited. Does not implicitly {@link usb_control.CmdAddWhitelist.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdAddWhitelist
         * @static
         * @param {usb_control.ICmdAddWhitelist} message CmdAddWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdAddWhitelist.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdAddWhitelist message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdAddWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdAddWhitelist} CmdAddWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdAddWhitelist.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdAddWhitelist();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.serialNumber = reader.string();
                        break;
                    }
                case 3: {
                        message.vid = reader.string();
                        break;
                    }
                case 4: {
                        message.pid = reader.string();
                        break;
                    }
                case 5: {
                        message.deviceName = reader.string();
                        break;
                    }
                case 6: {
                        message.capacityBytes = reader.int64();
                        break;
                    }
                case 7: {
                        message.permission = reader.string();
                        break;
                    }
                case 8: {
                        message.description = reader.string();
                        break;
                    }
                case 9: {
                        message.addMethod = reader.string();
                        break;
                    }
                case 10: {
                        message.deviceType = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdAddWhitelist message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdAddWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdAddWhitelist} CmdAddWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdAddWhitelist.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdAddWhitelist message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdAddWhitelist
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdAddWhitelist} CmdAddWhitelist
         */
        CmdAddWhitelist.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdAddWhitelist)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdAddWhitelist: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdAddWhitelist();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.serialNumber != null)
                message.serialNumber = String(object.serialNumber);
            if (object.vid != null)
                message.vid = String(object.vid);
            if (object.pid != null)
                message.pid = String(object.pid);
            if (object.deviceName != null)
                message.deviceName = String(object.deviceName);
            if (object.capacityBytes != null)
                if ($util.Long)
                    message.capacityBytes = $util.Long.fromValue(object.capacityBytes, false);
                else if (typeof object.capacityBytes === "string")
                    message.capacityBytes = parseInt(object.capacityBytes, 10);
                else if (typeof object.capacityBytes === "number")
                    message.capacityBytes = object.capacityBytes;
                else if (typeof object.capacityBytes === "object")
                    message.capacityBytes = new $util.LongBits(object.capacityBytes.low >>> 0, object.capacityBytes.high >>> 0).toNumber();
            if (object.permission != null)
                message.permission = String(object.permission);
            if (object.description != null)
                message.description = String(object.description);
            if (object.addMethod != null)
                message.addMethod = String(object.addMethod);
            if (object.deviceType != null)
                message.deviceType = String(object.deviceType);
            return message;
        };

        /**
         * Creates a plain object from a CmdAddWhitelist message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdAddWhitelist
         * @static
         * @param {usb_control.CmdAddWhitelist} message CmdAddWhitelist
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdAddWhitelist.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.serialNumber = "";
                object.vid = "";
                object.pid = "";
                object.deviceName = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.capacityBytes = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.capacityBytes = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.permission = "";
                object.description = "";
                object.addMethod = "";
                object.deviceType = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                object.serialNumber = message.serialNumber;
            if (message.vid != null && Object.hasOwnProperty.call(message, "vid"))
                object.vid = message.vid;
            if (message.pid != null && Object.hasOwnProperty.call(message, "pid"))
                object.pid = message.pid;
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                object.deviceName = message.deviceName;
            if (message.capacityBytes != null && Object.hasOwnProperty.call(message, "capacityBytes"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.capacityBytes = typeof message.capacityBytes === "number" ? BigInt(message.capacityBytes) : $util.Long.fromBits(message.capacityBytes.low >>> 0, message.capacityBytes.high >>> 0, false).toBigInt();
                else if (typeof message.capacityBytes === "number")
                    object.capacityBytes = options.longs === String ? String(message.capacityBytes) : message.capacityBytes;
                else
                    object.capacityBytes = options.longs === String ? $util.Long.prototype.toString.call(message.capacityBytes) : options.longs === Number ? new $util.LongBits(message.capacityBytes.low >>> 0, message.capacityBytes.high >>> 0).toNumber() : message.capacityBytes;
            if (message.permission != null && Object.hasOwnProperty.call(message, "permission"))
                object.permission = message.permission;
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                object.description = message.description;
            if (message.addMethod != null && Object.hasOwnProperty.call(message, "addMethod"))
                object.addMethod = message.addMethod;
            if (message.deviceType != null && Object.hasOwnProperty.call(message, "deviceType"))
                object.deviceType = message.deviceType;
            return object;
        };

        /**
         * Converts this CmdAddWhitelist to JSON.
         * @function toJSON
         * @memberof usb_control.CmdAddWhitelist
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdAddWhitelist.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdAddWhitelist
         * @function getTypeUrl
         * @memberof usb_control.CmdAddWhitelist
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdAddWhitelist.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdAddWhitelist";
        };

        return CmdAddWhitelist;
    })();

    usb_control.CmdRemoveWhitelist = (function() {

        /**
         * Properties of a CmdRemoveWhitelist.
         * @memberof usb_control
         * @interface ICmdRemoveWhitelist
         * @property {string|null} [sessionToken] CmdRemoveWhitelist sessionToken
         * @property {string|null} [serialNumber] CmdRemoveWhitelist serialNumber
         */

        /**
         * Constructs a new CmdRemoveWhitelist.
         * @memberof usb_control
         * @classdesc Represents a CmdRemoveWhitelist.
         * @implements ICmdRemoveWhitelist
         * @constructor
         * @param {usb_control.ICmdRemoveWhitelist=} [properties] Properties to set
         */
        function CmdRemoveWhitelist(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdRemoveWhitelist sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdRemoveWhitelist
         * @instance
         */
        CmdRemoveWhitelist.prototype.sessionToken = "";

        /**
         * CmdRemoveWhitelist serialNumber.
         * @member {string} serialNumber
         * @memberof usb_control.CmdRemoveWhitelist
         * @instance
         */
        CmdRemoveWhitelist.prototype.serialNumber = "";

        /**
         * Encodes the specified CmdRemoveWhitelist message. Does not implicitly {@link usb_control.CmdRemoveWhitelist.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdRemoveWhitelist
         * @static
         * @param {usb_control.ICmdRemoveWhitelist} message CmdRemoveWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdRemoveWhitelist.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.serialNumber);
            return writer;
        };

        /**
         * Encodes the specified CmdRemoveWhitelist message, length delimited. Does not implicitly {@link usb_control.CmdRemoveWhitelist.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdRemoveWhitelist
         * @static
         * @param {usb_control.ICmdRemoveWhitelist} message CmdRemoveWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdRemoveWhitelist.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdRemoveWhitelist message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdRemoveWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdRemoveWhitelist} CmdRemoveWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdRemoveWhitelist.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdRemoveWhitelist();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.serialNumber = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdRemoveWhitelist message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdRemoveWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdRemoveWhitelist} CmdRemoveWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdRemoveWhitelist.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdRemoveWhitelist message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdRemoveWhitelist
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdRemoveWhitelist} CmdRemoveWhitelist
         */
        CmdRemoveWhitelist.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdRemoveWhitelist)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdRemoveWhitelist: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdRemoveWhitelist();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.serialNumber != null)
                message.serialNumber = String(object.serialNumber);
            return message;
        };

        /**
         * Creates a plain object from a CmdRemoveWhitelist message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdRemoveWhitelist
         * @static
         * @param {usb_control.CmdRemoveWhitelist} message CmdRemoveWhitelist
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdRemoveWhitelist.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.serialNumber = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                object.serialNumber = message.serialNumber;
            return object;
        };

        /**
         * Converts this CmdRemoveWhitelist to JSON.
         * @function toJSON
         * @memberof usb_control.CmdRemoveWhitelist
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdRemoveWhitelist.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdRemoveWhitelist
         * @function getTypeUrl
         * @memberof usb_control.CmdRemoveWhitelist
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdRemoveWhitelist.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdRemoveWhitelist";
        };

        return CmdRemoveWhitelist;
    })();

    usb_control.CmdUpdateWhitelist = (function() {

        /**
         * Properties of a CmdUpdateWhitelist.
         * @memberof usb_control
         * @interface ICmdUpdateWhitelist
         * @property {string|null} [sessionToken] CmdUpdateWhitelist sessionToken
         * @property {string|null} [serialNumber] CmdUpdateWhitelist serialNumber
         * @property {string|null} [permission] CmdUpdateWhitelist permission
         * @property {string|null} [description] CmdUpdateWhitelist description
         */

        /**
         * Constructs a new CmdUpdateWhitelist.
         * @memberof usb_control
         * @classdesc Represents a CmdUpdateWhitelist.
         * @implements ICmdUpdateWhitelist
         * @constructor
         * @param {usb_control.ICmdUpdateWhitelist=} [properties] Properties to set
         */
        function CmdUpdateWhitelist(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdUpdateWhitelist sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdUpdateWhitelist
         * @instance
         */
        CmdUpdateWhitelist.prototype.sessionToken = "";

        /**
         * CmdUpdateWhitelist serialNumber.
         * @member {string} serialNumber
         * @memberof usb_control.CmdUpdateWhitelist
         * @instance
         */
        CmdUpdateWhitelist.prototype.serialNumber = "";

        /**
         * CmdUpdateWhitelist permission.
         * @member {string} permission
         * @memberof usb_control.CmdUpdateWhitelist
         * @instance
         */
        CmdUpdateWhitelist.prototype.permission = "";

        /**
         * CmdUpdateWhitelist description.
         * @member {string} description
         * @memberof usb_control.CmdUpdateWhitelist
         * @instance
         */
        CmdUpdateWhitelist.prototype.description = "";

        /**
         * Encodes the specified CmdUpdateWhitelist message. Does not implicitly {@link usb_control.CmdUpdateWhitelist.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdUpdateWhitelist
         * @static
         * @param {usb_control.ICmdUpdateWhitelist} message CmdUpdateWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUpdateWhitelist.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.serialNumber);
            if (message.permission != null && Object.hasOwnProperty.call(message, "permission"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.permission);
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.description);
            return writer;
        };

        /**
         * Encodes the specified CmdUpdateWhitelist message, length delimited. Does not implicitly {@link usb_control.CmdUpdateWhitelist.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdUpdateWhitelist
         * @static
         * @param {usb_control.ICmdUpdateWhitelist} message CmdUpdateWhitelist message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUpdateWhitelist.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdUpdateWhitelist message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdUpdateWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdUpdateWhitelist} CmdUpdateWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUpdateWhitelist.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdUpdateWhitelist();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.serialNumber = reader.string();
                        break;
                    }
                case 3: {
                        message.permission = reader.string();
                        break;
                    }
                case 4: {
                        message.description = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdUpdateWhitelist message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdUpdateWhitelist
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdUpdateWhitelist} CmdUpdateWhitelist
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUpdateWhitelist.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdUpdateWhitelist message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdUpdateWhitelist
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdUpdateWhitelist} CmdUpdateWhitelist
         */
        CmdUpdateWhitelist.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdUpdateWhitelist)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdUpdateWhitelist: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdUpdateWhitelist();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.serialNumber != null)
                message.serialNumber = String(object.serialNumber);
            if (object.permission != null)
                message.permission = String(object.permission);
            if (object.description != null)
                message.description = String(object.description);
            return message;
        };

        /**
         * Creates a plain object from a CmdUpdateWhitelist message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdUpdateWhitelist
         * @static
         * @param {usb_control.CmdUpdateWhitelist} message CmdUpdateWhitelist
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdUpdateWhitelist.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.serialNumber = "";
                object.permission = "";
                object.description = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.serialNumber != null && Object.hasOwnProperty.call(message, "serialNumber"))
                object.serialNumber = message.serialNumber;
            if (message.permission != null && Object.hasOwnProperty.call(message, "permission"))
                object.permission = message.permission;
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                object.description = message.description;
            return object;
        };

        /**
         * Converts this CmdUpdateWhitelist to JSON.
         * @function toJSON
         * @memberof usb_control.CmdUpdateWhitelist
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdUpdateWhitelist.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdUpdateWhitelist
         * @function getTypeUrl
         * @memberof usb_control.CmdUpdateWhitelist
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdUpdateWhitelist.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdUpdateWhitelist";
        };

        return CmdUpdateWhitelist;
    })();

    usb_control.FileTypeBlacklistItem = (function() {

        /**
         * Properties of a FileTypeBlacklistItem.
         * @memberof usb_control
         * @interface IFileTypeBlacklistItem
         * @property {string|null} [extension] FileTypeBlacklistItem extension
         * @property {string|null} [description] FileTypeBlacklistItem description
         * @property {boolean|null} [isDefault] FileTypeBlacklistItem isDefault
         * @property {number|Long|null} [createdAt] FileTypeBlacklistItem createdAt
         */

        /**
         * Constructs a new FileTypeBlacklistItem.
         * @memberof usb_control
         * @classdesc Represents a FileTypeBlacklistItem.
         * @implements IFileTypeBlacklistItem
         * @constructor
         * @param {usb_control.IFileTypeBlacklistItem=} [properties] Properties to set
         */
        function FileTypeBlacklistItem(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * FileTypeBlacklistItem extension.
         * @member {string} extension
         * @memberof usb_control.FileTypeBlacklistItem
         * @instance
         */
        FileTypeBlacklistItem.prototype.extension = "";

        /**
         * FileTypeBlacklistItem description.
         * @member {string} description
         * @memberof usb_control.FileTypeBlacklistItem
         * @instance
         */
        FileTypeBlacklistItem.prototype.description = "";

        /**
         * FileTypeBlacklistItem isDefault.
         * @member {boolean} isDefault
         * @memberof usb_control.FileTypeBlacklistItem
         * @instance
         */
        FileTypeBlacklistItem.prototype.isDefault = false;

        /**
         * FileTypeBlacklistItem createdAt.
         * @member {number|Long} createdAt
         * @memberof usb_control.FileTypeBlacklistItem
         * @instance
         */
        FileTypeBlacklistItem.prototype.createdAt = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * Encodes the specified FileTypeBlacklistItem message. Does not implicitly {@link usb_control.FileTypeBlacklistItem.verify|verify} messages.
         * @function encode
         * @memberof usb_control.FileTypeBlacklistItem
         * @static
         * @param {usb_control.IFileTypeBlacklistItem} message FileTypeBlacklistItem message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        FileTypeBlacklistItem.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.extension != null && Object.hasOwnProperty.call(message, "extension"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.extension);
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.description);
            if (message.isDefault != null && Object.hasOwnProperty.call(message, "isDefault"))
                writer.uint32(/* id 3, wireType 0 =*/24).bool(message.isDefault);
            if (message.createdAt != null && Object.hasOwnProperty.call(message, "createdAt"))
                writer.uint32(/* id 4, wireType 0 =*/32).int64(message.createdAt);
            return writer;
        };

        /**
         * Encodes the specified FileTypeBlacklistItem message, length delimited. Does not implicitly {@link usb_control.FileTypeBlacklistItem.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.FileTypeBlacklistItem
         * @static
         * @param {usb_control.IFileTypeBlacklistItem} message FileTypeBlacklistItem message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        FileTypeBlacklistItem.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a FileTypeBlacklistItem message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.FileTypeBlacklistItem
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.FileTypeBlacklistItem} FileTypeBlacklistItem
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        FileTypeBlacklistItem.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.FileTypeBlacklistItem();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.extension = reader.string();
                        break;
                    }
                case 2: {
                        message.description = reader.string();
                        break;
                    }
                case 3: {
                        message.isDefault = reader.bool();
                        break;
                    }
                case 4: {
                        message.createdAt = reader.int64();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a FileTypeBlacklistItem message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.FileTypeBlacklistItem
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.FileTypeBlacklistItem} FileTypeBlacklistItem
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        FileTypeBlacklistItem.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a FileTypeBlacklistItem message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.FileTypeBlacklistItem
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.FileTypeBlacklistItem} FileTypeBlacklistItem
         */
        FileTypeBlacklistItem.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.FileTypeBlacklistItem)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.FileTypeBlacklistItem: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.FileTypeBlacklistItem();
            if (object.extension != null)
                message.extension = String(object.extension);
            if (object.description != null)
                message.description = String(object.description);
            if (object.isDefault != null)
                message.isDefault = Boolean(object.isDefault);
            if (object.createdAt != null)
                if ($util.Long)
                    message.createdAt = $util.Long.fromValue(object.createdAt, false);
                else if (typeof object.createdAt === "string")
                    message.createdAt = parseInt(object.createdAt, 10);
                else if (typeof object.createdAt === "number")
                    message.createdAt = object.createdAt;
                else if (typeof object.createdAt === "object")
                    message.createdAt = new $util.LongBits(object.createdAt.low >>> 0, object.createdAt.high >>> 0).toNumber();
            return message;
        };

        /**
         * Creates a plain object from a FileTypeBlacklistItem message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.FileTypeBlacklistItem
         * @static
         * @param {usb_control.FileTypeBlacklistItem} message FileTypeBlacklistItem
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        FileTypeBlacklistItem.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.extension = "";
                object.description = "";
                object.isDefault = false;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.createdAt = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.createdAt = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
            }
            if (message.extension != null && Object.hasOwnProperty.call(message, "extension"))
                object.extension = message.extension;
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                object.description = message.description;
            if (message.isDefault != null && Object.hasOwnProperty.call(message, "isDefault"))
                object.isDefault = message.isDefault;
            if (message.createdAt != null && Object.hasOwnProperty.call(message, "createdAt"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.createdAt = typeof message.createdAt === "number" ? BigInt(message.createdAt) : $util.Long.fromBits(message.createdAt.low >>> 0, message.createdAt.high >>> 0, false).toBigInt();
                else if (typeof message.createdAt === "number")
                    object.createdAt = options.longs === String ? String(message.createdAt) : message.createdAt;
                else
                    object.createdAt = options.longs === String ? $util.Long.prototype.toString.call(message.createdAt) : options.longs === Number ? new $util.LongBits(message.createdAt.low >>> 0, message.createdAt.high >>> 0).toNumber() : message.createdAt;
            return object;
        };

        /**
         * Converts this FileTypeBlacklistItem to JSON.
         * @function toJSON
         * @memberof usb_control.FileTypeBlacklistItem
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        FileTypeBlacklistItem.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for FileTypeBlacklistItem
         * @function getTypeUrl
         * @memberof usb_control.FileTypeBlacklistItem
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        FileTypeBlacklistItem.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.FileTypeBlacklistItem";
        };

        return FileTypeBlacklistItem;
    })();

    usb_control.CmdGetFilePolicy = (function() {

        /**
         * Properties of a CmdGetFilePolicy.
         * @memberof usb_control
         * @interface ICmdGetFilePolicy
         * @property {string|null} [sessionToken] CmdGetFilePolicy sessionToken
         */

        /**
         * Constructs a new CmdGetFilePolicy.
         * @memberof usb_control
         * @classdesc Represents a CmdGetFilePolicy.
         * @implements ICmdGetFilePolicy
         * @constructor
         * @param {usb_control.ICmdGetFilePolicy=} [properties] Properties to set
         */
        function CmdGetFilePolicy(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdGetFilePolicy sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdGetFilePolicy
         * @instance
         */
        CmdGetFilePolicy.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdGetFilePolicy message. Does not implicitly {@link usb_control.CmdGetFilePolicy.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdGetFilePolicy
         * @static
         * @param {usb_control.ICmdGetFilePolicy} message CmdGetFilePolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdGetFilePolicy.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdGetFilePolicy message, length delimited. Does not implicitly {@link usb_control.CmdGetFilePolicy.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdGetFilePolicy
         * @static
         * @param {usb_control.ICmdGetFilePolicy} message CmdGetFilePolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdGetFilePolicy.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdGetFilePolicy message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdGetFilePolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdGetFilePolicy} CmdGetFilePolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdGetFilePolicy.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdGetFilePolicy();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdGetFilePolicy message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdGetFilePolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdGetFilePolicy} CmdGetFilePolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdGetFilePolicy.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdGetFilePolicy message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdGetFilePolicy
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdGetFilePolicy} CmdGetFilePolicy
         */
        CmdGetFilePolicy.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdGetFilePolicy)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdGetFilePolicy: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdGetFilePolicy();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdGetFilePolicy message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdGetFilePolicy
         * @static
         * @param {usb_control.CmdGetFilePolicy} message CmdGetFilePolicy
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdGetFilePolicy.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdGetFilePolicy to JSON.
         * @function toJSON
         * @memberof usb_control.CmdGetFilePolicy
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdGetFilePolicy.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdGetFilePolicy
         * @function getTypeUrl
         * @memberof usb_control.CmdGetFilePolicy
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdGetFilePolicy.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdGetFilePolicy";
        };

        return CmdGetFilePolicy;
    })();

    usb_control.RspFilePolicy = (function() {

        /**
         * Properties of a RspFilePolicy.
         * @memberof usb_control
         * @interface IRspFilePolicy
         * @property {boolean|null} [execControlEnabled] RspFilePolicy execControlEnabled
         * @property {boolean|null} [autoReadControlEnabled] RspFilePolicy autoReadControlEnabled
         * @property {boolean|null} [fileTypeBlacklistEnabled] RspFilePolicy fileTypeBlacklistEnabled
         * @property {Array.<usb_control.IFileTypeBlacklistItem>|null} [blacklist] RspFilePolicy blacklist
         */

        /**
         * Constructs a new RspFilePolicy.
         * @memberof usb_control
         * @classdesc Represents a RspFilePolicy.
         * @implements IRspFilePolicy
         * @constructor
         * @param {usb_control.IRspFilePolicy=} [properties] Properties to set
         */
        function RspFilePolicy(properties) {
            this.blacklist = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspFilePolicy execControlEnabled.
         * @member {boolean} execControlEnabled
         * @memberof usb_control.RspFilePolicy
         * @instance
         */
        RspFilePolicy.prototype.execControlEnabled = false;

        /**
         * RspFilePolicy autoReadControlEnabled.
         * @member {boolean} autoReadControlEnabled
         * @memberof usb_control.RspFilePolicy
         * @instance
         */
        RspFilePolicy.prototype.autoReadControlEnabled = false;

        /**
         * RspFilePolicy fileTypeBlacklistEnabled.
         * @member {boolean} fileTypeBlacklistEnabled
         * @memberof usb_control.RspFilePolicy
         * @instance
         */
        RspFilePolicy.prototype.fileTypeBlacklistEnabled = false;

        /**
         * RspFilePolicy blacklist.
         * @member {Array.<usb_control.IFileTypeBlacklistItem>} blacklist
         * @memberof usb_control.RspFilePolicy
         * @instance
         */
        RspFilePolicy.prototype.blacklist = $util.emptyArray;

        /**
         * Encodes the specified RspFilePolicy message. Does not implicitly {@link usb_control.RspFilePolicy.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspFilePolicy
         * @static
         * @param {usb_control.IRspFilePolicy} message RspFilePolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspFilePolicy.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.execControlEnabled != null && Object.hasOwnProperty.call(message, "execControlEnabled"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.execControlEnabled);
            if (message.autoReadControlEnabled != null && Object.hasOwnProperty.call(message, "autoReadControlEnabled"))
                writer.uint32(/* id 2, wireType 0 =*/16).bool(message.autoReadControlEnabled);
            if (message.fileTypeBlacklistEnabled != null && Object.hasOwnProperty.call(message, "fileTypeBlacklistEnabled"))
                writer.uint32(/* id 3, wireType 0 =*/24).bool(message.fileTypeBlacklistEnabled);
            if (message.blacklist != null && message.blacklist.length)
                for (let i = 0; i < message.blacklist.length; ++i)
                    $root.usb_control.FileTypeBlacklistItem.encode(message.blacklist[i], writer.uint32(/* id 4, wireType 2 =*/34).fork(), q + 1).ldelim();
            return writer;
        };

        /**
         * Encodes the specified RspFilePolicy message, length delimited. Does not implicitly {@link usb_control.RspFilePolicy.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspFilePolicy
         * @static
         * @param {usb_control.IRspFilePolicy} message RspFilePolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspFilePolicy.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspFilePolicy message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspFilePolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspFilePolicy} RspFilePolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspFilePolicy.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspFilePolicy();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.execControlEnabled = reader.bool();
                        break;
                    }
                case 2: {
                        message.autoReadControlEnabled = reader.bool();
                        break;
                    }
                case 3: {
                        message.fileTypeBlacklistEnabled = reader.bool();
                        break;
                    }
                case 4: {
                        if (!(message.blacklist && message.blacklist.length))
                            message.blacklist = [];
                        message.blacklist.push($root.usb_control.FileTypeBlacklistItem.decode(reader, reader.uint32(), undefined, long + 1));
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspFilePolicy message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspFilePolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspFilePolicy} RspFilePolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspFilePolicy.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspFilePolicy message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspFilePolicy
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspFilePolicy} RspFilePolicy
         */
        RspFilePolicy.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspFilePolicy)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspFilePolicy: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspFilePolicy();
            if (object.execControlEnabled != null)
                message.execControlEnabled = Boolean(object.execControlEnabled);
            if (object.autoReadControlEnabled != null)
                message.autoReadControlEnabled = Boolean(object.autoReadControlEnabled);
            if (object.fileTypeBlacklistEnabled != null)
                message.fileTypeBlacklistEnabled = Boolean(object.fileTypeBlacklistEnabled);
            if (object.blacklist) {
                if (!Array.isArray(object.blacklist))
                    throw TypeError(".usb_control.RspFilePolicy.blacklist: array expected");
                message.blacklist = [];
                for (let i = 0; i < object.blacklist.length; ++i) {
                    if (!$util.isObject(object.blacklist[i]))
                        throw TypeError(".usb_control.RspFilePolicy.blacklist: object expected");
                    message.blacklist[i] = $root.usb_control.FileTypeBlacklistItem.fromObject(object.blacklist[i], long + 1);
                }
            }
            return message;
        };

        /**
         * Creates a plain object from a RspFilePolicy message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspFilePolicy
         * @static
         * @param {usb_control.RspFilePolicy} message RspFilePolicy
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspFilePolicy.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.arrays || options.defaults)
                object.blacklist = [];
            if (options.defaults) {
                object.execControlEnabled = false;
                object.autoReadControlEnabled = false;
                object.fileTypeBlacklistEnabled = false;
            }
            if (message.execControlEnabled != null && Object.hasOwnProperty.call(message, "execControlEnabled"))
                object.execControlEnabled = message.execControlEnabled;
            if (message.autoReadControlEnabled != null && Object.hasOwnProperty.call(message, "autoReadControlEnabled"))
                object.autoReadControlEnabled = message.autoReadControlEnabled;
            if (message.fileTypeBlacklistEnabled != null && Object.hasOwnProperty.call(message, "fileTypeBlacklistEnabled"))
                object.fileTypeBlacklistEnabled = message.fileTypeBlacklistEnabled;
            if (message.blacklist && message.blacklist.length) {
                object.blacklist = [];
                for (let j = 0; j < message.blacklist.length; ++j)
                    object.blacklist[j] = $root.usb_control.FileTypeBlacklistItem.toObject(message.blacklist[j], options, q + 1);
            }
            return object;
        };

        /**
         * Converts this RspFilePolicy to JSON.
         * @function toJSON
         * @memberof usb_control.RspFilePolicy
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspFilePolicy.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspFilePolicy
         * @function getTypeUrl
         * @memberof usb_control.RspFilePolicy
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspFilePolicy.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspFilePolicy";
        };

        return RspFilePolicy;
    })();

    usb_control.CmdUpdateFilePolicySwitch = (function() {

        /**
         * Properties of a CmdUpdateFilePolicySwitch.
         * @memberof usb_control
         * @interface ICmdUpdateFilePolicySwitch
         * @property {string|null} [sessionToken] CmdUpdateFilePolicySwitch sessionToken
         * @property {string|null} [policyKey] CmdUpdateFilePolicySwitch policyKey
         * @property {boolean|null} [enabled] CmdUpdateFilePolicySwitch enabled
         */

        /**
         * Constructs a new CmdUpdateFilePolicySwitch.
         * @memberof usb_control
         * @classdesc Represents a CmdUpdateFilePolicySwitch.
         * @implements ICmdUpdateFilePolicySwitch
         * @constructor
         * @param {usb_control.ICmdUpdateFilePolicySwitch=} [properties] Properties to set
         */
        function CmdUpdateFilePolicySwitch(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdUpdateFilePolicySwitch sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @instance
         */
        CmdUpdateFilePolicySwitch.prototype.sessionToken = "";

        /**
         * CmdUpdateFilePolicySwitch policyKey.
         * @member {string} policyKey
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @instance
         */
        CmdUpdateFilePolicySwitch.prototype.policyKey = "";

        /**
         * CmdUpdateFilePolicySwitch enabled.
         * @member {boolean} enabled
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @instance
         */
        CmdUpdateFilePolicySwitch.prototype.enabled = false;

        /**
         * Encodes the specified CmdUpdateFilePolicySwitch message. Does not implicitly {@link usb_control.CmdUpdateFilePolicySwitch.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @static
         * @param {usb_control.ICmdUpdateFilePolicySwitch} message CmdUpdateFilePolicySwitch message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUpdateFilePolicySwitch.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.policyKey != null && Object.hasOwnProperty.call(message, "policyKey"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.policyKey);
            if (message.enabled != null && Object.hasOwnProperty.call(message, "enabled"))
                writer.uint32(/* id 3, wireType 0 =*/24).bool(message.enabled);
            return writer;
        };

        /**
         * Encodes the specified CmdUpdateFilePolicySwitch message, length delimited. Does not implicitly {@link usb_control.CmdUpdateFilePolicySwitch.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @static
         * @param {usb_control.ICmdUpdateFilePolicySwitch} message CmdUpdateFilePolicySwitch message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUpdateFilePolicySwitch.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdUpdateFilePolicySwitch message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdUpdateFilePolicySwitch} CmdUpdateFilePolicySwitch
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUpdateFilePolicySwitch.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdUpdateFilePolicySwitch();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.policyKey = reader.string();
                        break;
                    }
                case 3: {
                        message.enabled = reader.bool();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdUpdateFilePolicySwitch message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdUpdateFilePolicySwitch} CmdUpdateFilePolicySwitch
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUpdateFilePolicySwitch.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdUpdateFilePolicySwitch message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdUpdateFilePolicySwitch} CmdUpdateFilePolicySwitch
         */
        CmdUpdateFilePolicySwitch.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdUpdateFilePolicySwitch)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdUpdateFilePolicySwitch: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdUpdateFilePolicySwitch();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.policyKey != null)
                message.policyKey = String(object.policyKey);
            if (object.enabled != null)
                message.enabled = Boolean(object.enabled);
            return message;
        };

        /**
         * Creates a plain object from a CmdUpdateFilePolicySwitch message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @static
         * @param {usb_control.CmdUpdateFilePolicySwitch} message CmdUpdateFilePolicySwitch
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdUpdateFilePolicySwitch.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.policyKey = "";
                object.enabled = false;
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.policyKey != null && Object.hasOwnProperty.call(message, "policyKey"))
                object.policyKey = message.policyKey;
            if (message.enabled != null && Object.hasOwnProperty.call(message, "enabled"))
                object.enabled = message.enabled;
            return object;
        };

        /**
         * Converts this CmdUpdateFilePolicySwitch to JSON.
         * @function toJSON
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdUpdateFilePolicySwitch.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdUpdateFilePolicySwitch
         * @function getTypeUrl
         * @memberof usb_control.CmdUpdateFilePolicySwitch
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdUpdateFilePolicySwitch.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdUpdateFilePolicySwitch";
        };

        return CmdUpdateFilePolicySwitch;
    })();

    usb_control.CmdAddBlacklistExtension = (function() {

        /**
         * Properties of a CmdAddBlacklistExtension.
         * @memberof usb_control
         * @interface ICmdAddBlacklistExtension
         * @property {string|null} [sessionToken] CmdAddBlacklistExtension sessionToken
         * @property {string|null} [extension] CmdAddBlacklistExtension extension
         * @property {string|null} [description] CmdAddBlacklistExtension description
         */

        /**
         * Constructs a new CmdAddBlacklistExtension.
         * @memberof usb_control
         * @classdesc Represents a CmdAddBlacklistExtension.
         * @implements ICmdAddBlacklistExtension
         * @constructor
         * @param {usb_control.ICmdAddBlacklistExtension=} [properties] Properties to set
         */
        function CmdAddBlacklistExtension(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdAddBlacklistExtension sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdAddBlacklistExtension
         * @instance
         */
        CmdAddBlacklistExtension.prototype.sessionToken = "";

        /**
         * CmdAddBlacklistExtension extension.
         * @member {string} extension
         * @memberof usb_control.CmdAddBlacklistExtension
         * @instance
         */
        CmdAddBlacklistExtension.prototype.extension = "";

        /**
         * CmdAddBlacklistExtension description.
         * @member {string} description
         * @memberof usb_control.CmdAddBlacklistExtension
         * @instance
         */
        CmdAddBlacklistExtension.prototype.description = "";

        /**
         * Encodes the specified CmdAddBlacklistExtension message. Does not implicitly {@link usb_control.CmdAddBlacklistExtension.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdAddBlacklistExtension
         * @static
         * @param {usb_control.ICmdAddBlacklistExtension} message CmdAddBlacklistExtension message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdAddBlacklistExtension.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.extension != null && Object.hasOwnProperty.call(message, "extension"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.extension);
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.description);
            return writer;
        };

        /**
         * Encodes the specified CmdAddBlacklistExtension message, length delimited. Does not implicitly {@link usb_control.CmdAddBlacklistExtension.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdAddBlacklistExtension
         * @static
         * @param {usb_control.ICmdAddBlacklistExtension} message CmdAddBlacklistExtension message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdAddBlacklistExtension.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdAddBlacklistExtension message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdAddBlacklistExtension
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdAddBlacklistExtension} CmdAddBlacklistExtension
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdAddBlacklistExtension.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdAddBlacklistExtension();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.extension = reader.string();
                        break;
                    }
                case 3: {
                        message.description = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdAddBlacklistExtension message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdAddBlacklistExtension
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdAddBlacklistExtension} CmdAddBlacklistExtension
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdAddBlacklistExtension.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdAddBlacklistExtension message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdAddBlacklistExtension
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdAddBlacklistExtension} CmdAddBlacklistExtension
         */
        CmdAddBlacklistExtension.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdAddBlacklistExtension)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdAddBlacklistExtension: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdAddBlacklistExtension();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.extension != null)
                message.extension = String(object.extension);
            if (object.description != null)
                message.description = String(object.description);
            return message;
        };

        /**
         * Creates a plain object from a CmdAddBlacklistExtension message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdAddBlacklistExtension
         * @static
         * @param {usb_control.CmdAddBlacklistExtension} message CmdAddBlacklistExtension
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdAddBlacklistExtension.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.extension = "";
                object.description = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.extension != null && Object.hasOwnProperty.call(message, "extension"))
                object.extension = message.extension;
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                object.description = message.description;
            return object;
        };

        /**
         * Converts this CmdAddBlacklistExtension to JSON.
         * @function toJSON
         * @memberof usb_control.CmdAddBlacklistExtension
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdAddBlacklistExtension.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdAddBlacklistExtension
         * @function getTypeUrl
         * @memberof usb_control.CmdAddBlacklistExtension
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdAddBlacklistExtension.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdAddBlacklistExtension";
        };

        return CmdAddBlacklistExtension;
    })();

    usb_control.CmdRemoveBlacklistExtension = (function() {

        /**
         * Properties of a CmdRemoveBlacklistExtension.
         * @memberof usb_control
         * @interface ICmdRemoveBlacklistExtension
         * @property {string|null} [sessionToken] CmdRemoveBlacklistExtension sessionToken
         * @property {string|null} [extension] CmdRemoveBlacklistExtension extension
         */

        /**
         * Constructs a new CmdRemoveBlacklistExtension.
         * @memberof usb_control
         * @classdesc Represents a CmdRemoveBlacklistExtension.
         * @implements ICmdRemoveBlacklistExtension
         * @constructor
         * @param {usb_control.ICmdRemoveBlacklistExtension=} [properties] Properties to set
         */
        function CmdRemoveBlacklistExtension(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdRemoveBlacklistExtension sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @instance
         */
        CmdRemoveBlacklistExtension.prototype.sessionToken = "";

        /**
         * CmdRemoveBlacklistExtension extension.
         * @member {string} extension
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @instance
         */
        CmdRemoveBlacklistExtension.prototype.extension = "";

        /**
         * Encodes the specified CmdRemoveBlacklistExtension message. Does not implicitly {@link usb_control.CmdRemoveBlacklistExtension.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @static
         * @param {usb_control.ICmdRemoveBlacklistExtension} message CmdRemoveBlacklistExtension message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdRemoveBlacklistExtension.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.extension != null && Object.hasOwnProperty.call(message, "extension"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.extension);
            return writer;
        };

        /**
         * Encodes the specified CmdRemoveBlacklistExtension message, length delimited. Does not implicitly {@link usb_control.CmdRemoveBlacklistExtension.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @static
         * @param {usb_control.ICmdRemoveBlacklistExtension} message CmdRemoveBlacklistExtension message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdRemoveBlacklistExtension.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdRemoveBlacklistExtension message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdRemoveBlacklistExtension} CmdRemoveBlacklistExtension
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdRemoveBlacklistExtension.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdRemoveBlacklistExtension();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.extension = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdRemoveBlacklistExtension message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdRemoveBlacklistExtension} CmdRemoveBlacklistExtension
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdRemoveBlacklistExtension.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdRemoveBlacklistExtension message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdRemoveBlacklistExtension} CmdRemoveBlacklistExtension
         */
        CmdRemoveBlacklistExtension.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdRemoveBlacklistExtension)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdRemoveBlacklistExtension: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdRemoveBlacklistExtension();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.extension != null)
                message.extension = String(object.extension);
            return message;
        };

        /**
         * Creates a plain object from a CmdRemoveBlacklistExtension message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @static
         * @param {usb_control.CmdRemoveBlacklistExtension} message CmdRemoveBlacklistExtension
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdRemoveBlacklistExtension.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.extension = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.extension != null && Object.hasOwnProperty.call(message, "extension"))
                object.extension = message.extension;
            return object;
        };

        /**
         * Converts this CmdRemoveBlacklistExtension to JSON.
         * @function toJSON
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdRemoveBlacklistExtension.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdRemoveBlacklistExtension
         * @function getTypeUrl
         * @memberof usb_control.CmdRemoveBlacklistExtension
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdRemoveBlacklistExtension.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdRemoveBlacklistExtension";
        };

        return CmdRemoveBlacklistExtension;
    })();

    usb_control.CmdExportPolicy = (function() {

        /**
         * Properties of a CmdExportPolicy.
         * @memberof usb_control
         * @interface ICmdExportPolicy
         * @property {string|null} [sessionToken] CmdExportPolicy sessionToken
         */

        /**
         * Constructs a new CmdExportPolicy.
         * @memberof usb_control
         * @classdesc Represents a CmdExportPolicy.
         * @implements ICmdExportPolicy
         * @constructor
         * @param {usb_control.ICmdExportPolicy=} [properties] Properties to set
         */
        function CmdExportPolicy(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdExportPolicy sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdExportPolicy
         * @instance
         */
        CmdExportPolicy.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdExportPolicy message. Does not implicitly {@link usb_control.CmdExportPolicy.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdExportPolicy
         * @static
         * @param {usb_control.ICmdExportPolicy} message CmdExportPolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdExportPolicy.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdExportPolicy message, length delimited. Does not implicitly {@link usb_control.CmdExportPolicy.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdExportPolicy
         * @static
         * @param {usb_control.ICmdExportPolicy} message CmdExportPolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdExportPolicy.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdExportPolicy message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdExportPolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdExportPolicy} CmdExportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdExportPolicy.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdExportPolicy();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdExportPolicy message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdExportPolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdExportPolicy} CmdExportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdExportPolicy.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdExportPolicy message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdExportPolicy
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdExportPolicy} CmdExportPolicy
         */
        CmdExportPolicy.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdExportPolicy)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdExportPolicy: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdExportPolicy();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdExportPolicy message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdExportPolicy
         * @static
         * @param {usb_control.CmdExportPolicy} message CmdExportPolicy
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdExportPolicy.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdExportPolicy to JSON.
         * @function toJSON
         * @memberof usb_control.CmdExportPolicy
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdExportPolicy.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdExportPolicy
         * @function getTypeUrl
         * @memberof usb_control.CmdExportPolicy
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdExportPolicy.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdExportPolicy";
        };

        return CmdExportPolicy;
    })();

    usb_control.RspExportPolicy = (function() {

        /**
         * Properties of a RspExportPolicy.
         * @memberof usb_control
         * @interface IRspExportPolicy
         * @property {boolean|null} [success] RspExportPolicy success
         * @property {Uint8Array|null} [policyData] RspExportPolicy policyData
         * @property {number|null} [resultCode] RspExportPolicy resultCode
         * @property {string|null} [errorMessage] RspExportPolicy errorMessage
         */

        /**
         * Constructs a new RspExportPolicy.
         * @memberof usb_control
         * @classdesc Represents a RspExportPolicy.
         * @implements IRspExportPolicy
         * @constructor
         * @param {usb_control.IRspExportPolicy=} [properties] Properties to set
         */
        function RspExportPolicy(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspExportPolicy success.
         * @member {boolean} success
         * @memberof usb_control.RspExportPolicy
         * @instance
         */
        RspExportPolicy.prototype.success = false;

        /**
         * RspExportPolicy policyData.
         * @member {Uint8Array} policyData
         * @memberof usb_control.RspExportPolicy
         * @instance
         */
        RspExportPolicy.prototype.policyData = $util.newBuffer([]);

        /**
         * RspExportPolicy resultCode.
         * @member {number} resultCode
         * @memberof usb_control.RspExportPolicy
         * @instance
         */
        RspExportPolicy.prototype.resultCode = 0;

        /**
         * RspExportPolicy errorMessage.
         * @member {string} errorMessage
         * @memberof usb_control.RspExportPolicy
         * @instance
         */
        RspExportPolicy.prototype.errorMessage = "";

        /**
         * Encodes the specified RspExportPolicy message. Does not implicitly {@link usb_control.RspExportPolicy.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspExportPolicy
         * @static
         * @param {usb_control.IRspExportPolicy} message RspExportPolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspExportPolicy.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
            if (message.policyData != null && Object.hasOwnProperty.call(message, "policyData"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.policyData);
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                writer.uint32(/* id 3, wireType 0 =*/24).int32(message.resultCode);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.errorMessage);
            return writer;
        };

        /**
         * Encodes the specified RspExportPolicy message, length delimited. Does not implicitly {@link usb_control.RspExportPolicy.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspExportPolicy
         * @static
         * @param {usb_control.IRspExportPolicy} message RspExportPolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspExportPolicy.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspExportPolicy message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspExportPolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspExportPolicy} RspExportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspExportPolicy.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspExportPolicy();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.success = reader.bool();
                        break;
                    }
                case 2: {
                        message.policyData = reader.bytes();
                        break;
                    }
                case 3: {
                        message.resultCode = reader.int32();
                        break;
                    }
                case 4: {
                        message.errorMessage = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspExportPolicy message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspExportPolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspExportPolicy} RspExportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspExportPolicy.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspExportPolicy message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspExportPolicy
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspExportPolicy} RspExportPolicy
         */
        RspExportPolicy.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspExportPolicy)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspExportPolicy: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspExportPolicy();
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.policyData != null)
                if (typeof object.policyData === "string")
                    $util.base64.decode(object.policyData, message.policyData = $util.newBuffer($util.base64.length(object.policyData)), 0);
                else if (object.policyData.length >= 0)
                    message.policyData = object.policyData;
            if (object.resultCode != null)
                message.resultCode = object.resultCode | 0;
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            return message;
        };

        /**
         * Creates a plain object from a RspExportPolicy message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspExportPolicy
         * @static
         * @param {usb_control.RspExportPolicy} message RspExportPolicy
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspExportPolicy.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.success = false;
                if (options.bytes === String)
                    object.policyData = "";
                else {
                    object.policyData = [];
                    if (options.bytes !== Array)
                        object.policyData = $util.newBuffer(object.policyData);
                }
                object.resultCode = 0;
                object.errorMessage = "";
            }
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                object.success = message.success;
            if (message.policyData != null && Object.hasOwnProperty.call(message, "policyData"))
                object.policyData = options.bytes === String ? $util.base64.encode(message.policyData, 0, message.policyData.length) : options.bytes === Array ? Array.prototype.slice.call(message.policyData) : message.policyData;
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                object.resultCode = message.resultCode;
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                object.errorMessage = message.errorMessage;
            return object;
        };

        /**
         * Converts this RspExportPolicy to JSON.
         * @function toJSON
         * @memberof usb_control.RspExportPolicy
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspExportPolicy.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspExportPolicy
         * @function getTypeUrl
         * @memberof usb_control.RspExportPolicy
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspExportPolicy.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspExportPolicy";
        };

        return RspExportPolicy;
    })();

    usb_control.CmdImportPolicy = (function() {

        /**
         * Properties of a CmdImportPolicy.
         * @memberof usb_control
         * @interface ICmdImportPolicy
         * @property {string|null} [sessionToken] CmdImportPolicy sessionToken
         * @property {Uint8Array|null} [policyData] CmdImportPolicy policyData
         */

        /**
         * Constructs a new CmdImportPolicy.
         * @memberof usb_control
         * @classdesc Represents a CmdImportPolicy.
         * @implements ICmdImportPolicy
         * @constructor
         * @param {usb_control.ICmdImportPolicy=} [properties] Properties to set
         */
        function CmdImportPolicy(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdImportPolicy sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdImportPolicy
         * @instance
         */
        CmdImportPolicy.prototype.sessionToken = "";

        /**
         * CmdImportPolicy policyData.
         * @member {Uint8Array} policyData
         * @memberof usb_control.CmdImportPolicy
         * @instance
         */
        CmdImportPolicy.prototype.policyData = $util.newBuffer([]);

        /**
         * Encodes the specified CmdImportPolicy message. Does not implicitly {@link usb_control.CmdImportPolicy.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdImportPolicy
         * @static
         * @param {usb_control.ICmdImportPolicy} message CmdImportPolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdImportPolicy.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.policyData != null && Object.hasOwnProperty.call(message, "policyData"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.policyData);
            return writer;
        };

        /**
         * Encodes the specified CmdImportPolicy message, length delimited. Does not implicitly {@link usb_control.CmdImportPolicy.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdImportPolicy
         * @static
         * @param {usb_control.ICmdImportPolicy} message CmdImportPolicy message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdImportPolicy.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdImportPolicy message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdImportPolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdImportPolicy} CmdImportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdImportPolicy.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdImportPolicy();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.policyData = reader.bytes();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdImportPolicy message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdImportPolicy
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdImportPolicy} CmdImportPolicy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdImportPolicy.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdImportPolicy message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdImportPolicy
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdImportPolicy} CmdImportPolicy
         */
        CmdImportPolicy.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdImportPolicy)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdImportPolicy: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdImportPolicy();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.policyData != null)
                if (typeof object.policyData === "string")
                    $util.base64.decode(object.policyData, message.policyData = $util.newBuffer($util.base64.length(object.policyData)), 0);
                else if (object.policyData.length >= 0)
                    message.policyData = object.policyData;
            return message;
        };

        /**
         * Creates a plain object from a CmdImportPolicy message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdImportPolicy
         * @static
         * @param {usb_control.CmdImportPolicy} message CmdImportPolicy
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdImportPolicy.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                if (options.bytes === String)
                    object.policyData = "";
                else {
                    object.policyData = [];
                    if (options.bytes !== Array)
                        object.policyData = $util.newBuffer(object.policyData);
                }
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.policyData != null && Object.hasOwnProperty.call(message, "policyData"))
                object.policyData = options.bytes === String ? $util.base64.encode(message.policyData, 0, message.policyData.length) : options.bytes === Array ? Array.prototype.slice.call(message.policyData) : message.policyData;
            return object;
        };

        /**
         * Converts this CmdImportPolicy to JSON.
         * @function toJSON
         * @memberof usb_control.CmdImportPolicy
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdImportPolicy.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdImportPolicy
         * @function getTypeUrl
         * @memberof usb_control.CmdImportPolicy
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdImportPolicy.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdImportPolicy";
        };

        return CmdImportPolicy;
    })();

    usb_control.CmdQueryLogs = (function() {

        /**
         * Properties of a CmdQueryLogs.
         * @memberof usb_control
         * @interface ICmdQueryLogs
         * @property {string|null} [sessionToken] CmdQueryLogs sessionToken
         * @property {string|null} [logType] CmdQueryLogs logType
         * @property {number|Long|null} [startTime] CmdQueryLogs startTime
         * @property {number|Long|null} [endTime] CmdQueryLogs endTime
         * @property {string|null} [keyword] CmdQueryLogs keyword
         * @property {string|null} [eventType] CmdQueryLogs eventType
         * @property {number|null} [page] CmdQueryLogs page
         * @property {number|null} [pageSize] CmdQueryLogs pageSize
         */

        /**
         * Constructs a new CmdQueryLogs.
         * @memberof usb_control
         * @classdesc Represents a CmdQueryLogs.
         * @implements ICmdQueryLogs
         * @constructor
         * @param {usb_control.ICmdQueryLogs=} [properties] Properties to set
         */
        function CmdQueryLogs(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdQueryLogs sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdQueryLogs
         * @instance
         */
        CmdQueryLogs.prototype.sessionToken = "";

        /**
         * CmdQueryLogs logType.
         * @member {string} logType
         * @memberof usb_control.CmdQueryLogs
         * @instance
         */
        CmdQueryLogs.prototype.logType = "";

        /**
         * CmdQueryLogs startTime.
         * @member {number|Long} startTime
         * @memberof usb_control.CmdQueryLogs
         * @instance
         */
        CmdQueryLogs.prototype.startTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * CmdQueryLogs endTime.
         * @member {number|Long} endTime
         * @memberof usb_control.CmdQueryLogs
         * @instance
         */
        CmdQueryLogs.prototype.endTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * CmdQueryLogs keyword.
         * @member {string} keyword
         * @memberof usb_control.CmdQueryLogs
         * @instance
         */
        CmdQueryLogs.prototype.keyword = "";

        /**
         * CmdQueryLogs eventType.
         * @member {string} eventType
         * @memberof usb_control.CmdQueryLogs
         * @instance
         */
        CmdQueryLogs.prototype.eventType = "";

        /**
         * CmdQueryLogs page.
         * @member {number} page
         * @memberof usb_control.CmdQueryLogs
         * @instance
         */
        CmdQueryLogs.prototype.page = 0;

        /**
         * CmdQueryLogs pageSize.
         * @member {number} pageSize
         * @memberof usb_control.CmdQueryLogs
         * @instance
         */
        CmdQueryLogs.prototype.pageSize = 0;

        /**
         * Encodes the specified CmdQueryLogs message. Does not implicitly {@link usb_control.CmdQueryLogs.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdQueryLogs
         * @static
         * @param {usb_control.ICmdQueryLogs} message CmdQueryLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdQueryLogs.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.logType != null && Object.hasOwnProperty.call(message, "logType"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.logType);
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                writer.uint32(/* id 3, wireType 0 =*/24).int64(message.startTime);
            if (message.endTime != null && Object.hasOwnProperty.call(message, "endTime"))
                writer.uint32(/* id 4, wireType 0 =*/32).int64(message.endTime);
            if (message.keyword != null && Object.hasOwnProperty.call(message, "keyword"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.keyword);
            if (message.eventType != null && Object.hasOwnProperty.call(message, "eventType"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.eventType);
            if (message.page != null && Object.hasOwnProperty.call(message, "page"))
                writer.uint32(/* id 7, wireType 0 =*/56).int32(message.page);
            if (message.pageSize != null && Object.hasOwnProperty.call(message, "pageSize"))
                writer.uint32(/* id 8, wireType 0 =*/64).int32(message.pageSize);
            return writer;
        };

        /**
         * Encodes the specified CmdQueryLogs message, length delimited. Does not implicitly {@link usb_control.CmdQueryLogs.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdQueryLogs
         * @static
         * @param {usb_control.ICmdQueryLogs} message CmdQueryLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdQueryLogs.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdQueryLogs message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdQueryLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdQueryLogs} CmdQueryLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdQueryLogs.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdQueryLogs();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.logType = reader.string();
                        break;
                    }
                case 3: {
                        message.startTime = reader.int64();
                        break;
                    }
                case 4: {
                        message.endTime = reader.int64();
                        break;
                    }
                case 5: {
                        message.keyword = reader.string();
                        break;
                    }
                case 6: {
                        message.eventType = reader.string();
                        break;
                    }
                case 7: {
                        message.page = reader.int32();
                        break;
                    }
                case 8: {
                        message.pageSize = reader.int32();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdQueryLogs message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdQueryLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdQueryLogs} CmdQueryLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdQueryLogs.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdQueryLogs message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdQueryLogs
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdQueryLogs} CmdQueryLogs
         */
        CmdQueryLogs.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdQueryLogs)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdQueryLogs: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdQueryLogs();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.logType != null)
                message.logType = String(object.logType);
            if (object.startTime != null)
                if ($util.Long)
                    message.startTime = $util.Long.fromValue(object.startTime, false);
                else if (typeof object.startTime === "string")
                    message.startTime = parseInt(object.startTime, 10);
                else if (typeof object.startTime === "number")
                    message.startTime = object.startTime;
                else if (typeof object.startTime === "object")
                    message.startTime = new $util.LongBits(object.startTime.low >>> 0, object.startTime.high >>> 0).toNumber();
            if (object.endTime != null)
                if ($util.Long)
                    message.endTime = $util.Long.fromValue(object.endTime, false);
                else if (typeof object.endTime === "string")
                    message.endTime = parseInt(object.endTime, 10);
                else if (typeof object.endTime === "number")
                    message.endTime = object.endTime;
                else if (typeof object.endTime === "object")
                    message.endTime = new $util.LongBits(object.endTime.low >>> 0, object.endTime.high >>> 0).toNumber();
            if (object.keyword != null)
                message.keyword = String(object.keyword);
            if (object.eventType != null)
                message.eventType = String(object.eventType);
            if (object.page != null)
                message.page = object.page | 0;
            if (object.pageSize != null)
                message.pageSize = object.pageSize | 0;
            return message;
        };

        /**
         * Creates a plain object from a CmdQueryLogs message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdQueryLogs
         * @static
         * @param {usb_control.CmdQueryLogs} message CmdQueryLogs
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdQueryLogs.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.logType = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.startTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.startTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.endTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.endTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.keyword = "";
                object.eventType = "";
                object.page = 0;
                object.pageSize = 0;
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.logType != null && Object.hasOwnProperty.call(message, "logType"))
                object.logType = message.logType;
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.startTime = typeof message.startTime === "number" ? BigInt(message.startTime) : $util.Long.fromBits(message.startTime.low >>> 0, message.startTime.high >>> 0, false).toBigInt();
                else if (typeof message.startTime === "number")
                    object.startTime = options.longs === String ? String(message.startTime) : message.startTime;
                else
                    object.startTime = options.longs === String ? $util.Long.prototype.toString.call(message.startTime) : options.longs === Number ? new $util.LongBits(message.startTime.low >>> 0, message.startTime.high >>> 0).toNumber() : message.startTime;
            if (message.endTime != null && Object.hasOwnProperty.call(message, "endTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.endTime = typeof message.endTime === "number" ? BigInt(message.endTime) : $util.Long.fromBits(message.endTime.low >>> 0, message.endTime.high >>> 0, false).toBigInt();
                else if (typeof message.endTime === "number")
                    object.endTime = options.longs === String ? String(message.endTime) : message.endTime;
                else
                    object.endTime = options.longs === String ? $util.Long.prototype.toString.call(message.endTime) : options.longs === Number ? new $util.LongBits(message.endTime.low >>> 0, message.endTime.high >>> 0).toNumber() : message.endTime;
            if (message.keyword != null && Object.hasOwnProperty.call(message, "keyword"))
                object.keyword = message.keyword;
            if (message.eventType != null && Object.hasOwnProperty.call(message, "eventType"))
                object.eventType = message.eventType;
            if (message.page != null && Object.hasOwnProperty.call(message, "page"))
                object.page = message.page;
            if (message.pageSize != null && Object.hasOwnProperty.call(message, "pageSize"))
                object.pageSize = message.pageSize;
            return object;
        };

        /**
         * Converts this CmdQueryLogs to JSON.
         * @function toJSON
         * @memberof usb_control.CmdQueryLogs
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdQueryLogs.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdQueryLogs
         * @function getTypeUrl
         * @memberof usb_control.CmdQueryLogs
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdQueryLogs.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdQueryLogs";
        };

        return CmdQueryLogs;
    })();

    usb_control.UsbAuditLogEntry = (function() {

        /**
         * Properties of a UsbAuditLogEntry.
         * @memberof usb_control
         * @interface IUsbAuditLogEntry
         * @property {number|Long|null} [id] UsbAuditLogEntry id
         * @property {number|Long|null} [eventTime] UsbAuditLogEntry eventTime
         * @property {string|null} [deviceSn] UsbAuditLogEntry deviceSn
         * @property {string|null} [deviceName] UsbAuditLogEntry deviceName
         * @property {string|null} [deviceType] UsbAuditLogEntry deviceType
         * @property {string|null} [interfaceType] UsbAuditLogEntry interfaceType
         * @property {string|null} [eventType] UsbAuditLogEntry eventType
         * @property {string|null} [permission] UsbAuditLogEntry permission
         * @property {number|Long|null} [capacityBytes] UsbAuditLogEntry capacityBytes
         * @property {string|null} [detail] UsbAuditLogEntry detail
         */

        /**
         * Constructs a new UsbAuditLogEntry.
         * @memberof usb_control
         * @classdesc Represents a UsbAuditLogEntry.
         * @implements IUsbAuditLogEntry
         * @constructor
         * @param {usb_control.IUsbAuditLogEntry=} [properties] Properties to set
         */
        function UsbAuditLogEntry(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * UsbAuditLogEntry id.
         * @member {number|Long} id
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.id = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * UsbAuditLogEntry eventTime.
         * @member {number|Long} eventTime
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.eventTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * UsbAuditLogEntry deviceSn.
         * @member {string} deviceSn
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.deviceSn = "";

        /**
         * UsbAuditLogEntry deviceName.
         * @member {string} deviceName
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.deviceName = "";

        /**
         * UsbAuditLogEntry deviceType.
         * @member {string} deviceType
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.deviceType = "";

        /**
         * UsbAuditLogEntry interfaceType.
         * @member {string} interfaceType
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.interfaceType = "";

        /**
         * UsbAuditLogEntry eventType.
         * @member {string} eventType
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.eventType = "";

        /**
         * UsbAuditLogEntry permission.
         * @member {string} permission
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.permission = "";

        /**
         * UsbAuditLogEntry capacityBytes.
         * @member {number|Long} capacityBytes
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.capacityBytes = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * UsbAuditLogEntry detail.
         * @member {string} detail
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         */
        UsbAuditLogEntry.prototype.detail = "";

        /**
         * Encodes the specified UsbAuditLogEntry message. Does not implicitly {@link usb_control.UsbAuditLogEntry.verify|verify} messages.
         * @function encode
         * @memberof usb_control.UsbAuditLogEntry
         * @static
         * @param {usb_control.IUsbAuditLogEntry} message UsbAuditLogEntry message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        UsbAuditLogEntry.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 0 =*/8).int64(message.id);
            if (message.eventTime != null && Object.hasOwnProperty.call(message, "eventTime"))
                writer.uint32(/* id 2, wireType 0 =*/16).int64(message.eventTime);
            if (message.deviceSn != null && Object.hasOwnProperty.call(message, "deviceSn"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.deviceSn);
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.deviceName);
            if (message.deviceType != null && Object.hasOwnProperty.call(message, "deviceType"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.deviceType);
            if (message.interfaceType != null && Object.hasOwnProperty.call(message, "interfaceType"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.interfaceType);
            if (message.eventType != null && Object.hasOwnProperty.call(message, "eventType"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.eventType);
            if (message.permission != null && Object.hasOwnProperty.call(message, "permission"))
                writer.uint32(/* id 8, wireType 2 =*/66).string(message.permission);
            if (message.capacityBytes != null && Object.hasOwnProperty.call(message, "capacityBytes"))
                writer.uint32(/* id 9, wireType 0 =*/72).int64(message.capacityBytes);
            if (message.detail != null && Object.hasOwnProperty.call(message, "detail"))
                writer.uint32(/* id 14, wireType 2 =*/114).string(message.detail);
            return writer;
        };

        /**
         * Encodes the specified UsbAuditLogEntry message, length delimited. Does not implicitly {@link usb_control.UsbAuditLogEntry.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.UsbAuditLogEntry
         * @static
         * @param {usb_control.IUsbAuditLogEntry} message UsbAuditLogEntry message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        UsbAuditLogEntry.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a UsbAuditLogEntry message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.UsbAuditLogEntry
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.UsbAuditLogEntry} UsbAuditLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        UsbAuditLogEntry.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.UsbAuditLogEntry();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.id = reader.int64();
                        break;
                    }
                case 2: {
                        message.eventTime = reader.int64();
                        break;
                    }
                case 3: {
                        message.deviceSn = reader.string();
                        break;
                    }
                case 4: {
                        message.deviceName = reader.string();
                        break;
                    }
                case 5: {
                        message.deviceType = reader.string();
                        break;
                    }
                case 6: {
                        message.interfaceType = reader.string();
                        break;
                    }
                case 7: {
                        message.eventType = reader.string();
                        break;
                    }
                case 8: {
                        message.permission = reader.string();
                        break;
                    }
                case 9: {
                        message.capacityBytes = reader.int64();
                        break;
                    }
                case 14: {
                        message.detail = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a UsbAuditLogEntry message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.UsbAuditLogEntry
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.UsbAuditLogEntry} UsbAuditLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        UsbAuditLogEntry.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a UsbAuditLogEntry message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.UsbAuditLogEntry
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.UsbAuditLogEntry} UsbAuditLogEntry
         */
        UsbAuditLogEntry.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.UsbAuditLogEntry)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.UsbAuditLogEntry: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.UsbAuditLogEntry();
            if (object.id != null)
                if ($util.Long)
                    message.id = $util.Long.fromValue(object.id, false);
                else if (typeof object.id === "string")
                    message.id = parseInt(object.id, 10);
                else if (typeof object.id === "number")
                    message.id = object.id;
                else if (typeof object.id === "object")
                    message.id = new $util.LongBits(object.id.low >>> 0, object.id.high >>> 0).toNumber();
            if (object.eventTime != null)
                if ($util.Long)
                    message.eventTime = $util.Long.fromValue(object.eventTime, false);
                else if (typeof object.eventTime === "string")
                    message.eventTime = parseInt(object.eventTime, 10);
                else if (typeof object.eventTime === "number")
                    message.eventTime = object.eventTime;
                else if (typeof object.eventTime === "object")
                    message.eventTime = new $util.LongBits(object.eventTime.low >>> 0, object.eventTime.high >>> 0).toNumber();
            if (object.deviceSn != null)
                message.deviceSn = String(object.deviceSn);
            if (object.deviceName != null)
                message.deviceName = String(object.deviceName);
            if (object.deviceType != null)
                message.deviceType = String(object.deviceType);
            if (object.interfaceType != null)
                message.interfaceType = String(object.interfaceType);
            if (object.eventType != null)
                message.eventType = String(object.eventType);
            if (object.permission != null)
                message.permission = String(object.permission);
            if (object.capacityBytes != null)
                if ($util.Long)
                    message.capacityBytes = $util.Long.fromValue(object.capacityBytes, false);
                else if (typeof object.capacityBytes === "string")
                    message.capacityBytes = parseInt(object.capacityBytes, 10);
                else if (typeof object.capacityBytes === "number")
                    message.capacityBytes = object.capacityBytes;
                else if (typeof object.capacityBytes === "object")
                    message.capacityBytes = new $util.LongBits(object.capacityBytes.low >>> 0, object.capacityBytes.high >>> 0).toNumber();
            if (object.detail != null)
                message.detail = String(object.detail);
            return message;
        };

        /**
         * Creates a plain object from a UsbAuditLogEntry message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.UsbAuditLogEntry
         * @static
         * @param {usb_control.UsbAuditLogEntry} message UsbAuditLogEntry
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        UsbAuditLogEntry.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.id = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.id = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.eventTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.eventTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.deviceSn = "";
                object.deviceName = "";
                object.deviceType = "";
                object.interfaceType = "";
                object.eventType = "";
                object.permission = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.capacityBytes = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.capacityBytes = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.detail = "";
            }
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.id = typeof message.id === "number" ? BigInt(message.id) : $util.Long.fromBits(message.id.low >>> 0, message.id.high >>> 0, false).toBigInt();
                else if (typeof message.id === "number")
                    object.id = options.longs === String ? String(message.id) : message.id;
                else
                    object.id = options.longs === String ? $util.Long.prototype.toString.call(message.id) : options.longs === Number ? new $util.LongBits(message.id.low >>> 0, message.id.high >>> 0).toNumber() : message.id;
            if (message.eventTime != null && Object.hasOwnProperty.call(message, "eventTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.eventTime = typeof message.eventTime === "number" ? BigInt(message.eventTime) : $util.Long.fromBits(message.eventTime.low >>> 0, message.eventTime.high >>> 0, false).toBigInt();
                else if (typeof message.eventTime === "number")
                    object.eventTime = options.longs === String ? String(message.eventTime) : message.eventTime;
                else
                    object.eventTime = options.longs === String ? $util.Long.prototype.toString.call(message.eventTime) : options.longs === Number ? new $util.LongBits(message.eventTime.low >>> 0, message.eventTime.high >>> 0).toNumber() : message.eventTime;
            if (message.deviceSn != null && Object.hasOwnProperty.call(message, "deviceSn"))
                object.deviceSn = message.deviceSn;
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                object.deviceName = message.deviceName;
            if (message.deviceType != null && Object.hasOwnProperty.call(message, "deviceType"))
                object.deviceType = message.deviceType;
            if (message.interfaceType != null && Object.hasOwnProperty.call(message, "interfaceType"))
                object.interfaceType = message.interfaceType;
            if (message.eventType != null && Object.hasOwnProperty.call(message, "eventType"))
                object.eventType = message.eventType;
            if (message.permission != null && Object.hasOwnProperty.call(message, "permission"))
                object.permission = message.permission;
            if (message.capacityBytes != null && Object.hasOwnProperty.call(message, "capacityBytes"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.capacityBytes = typeof message.capacityBytes === "number" ? BigInt(message.capacityBytes) : $util.Long.fromBits(message.capacityBytes.low >>> 0, message.capacityBytes.high >>> 0, false).toBigInt();
                else if (typeof message.capacityBytes === "number")
                    object.capacityBytes = options.longs === String ? String(message.capacityBytes) : message.capacityBytes;
                else
                    object.capacityBytes = options.longs === String ? $util.Long.prototype.toString.call(message.capacityBytes) : options.longs === Number ? new $util.LongBits(message.capacityBytes.low >>> 0, message.capacityBytes.high >>> 0).toNumber() : message.capacityBytes;
            if (message.detail != null && Object.hasOwnProperty.call(message, "detail"))
                object.detail = message.detail;
            return object;
        };

        /**
         * Converts this UsbAuditLogEntry to JSON.
         * @function toJSON
         * @memberof usb_control.UsbAuditLogEntry
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        UsbAuditLogEntry.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for UsbAuditLogEntry
         * @function getTypeUrl
         * @memberof usb_control.UsbAuditLogEntry
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        UsbAuditLogEntry.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.UsbAuditLogEntry";
        };

        return UsbAuditLogEntry;
    })();

    usb_control.MalwareLogEntry = (function() {

        /**
         * Properties of a MalwareLogEntry.
         * @memberof usb_control
         * @interface IMalwareLogEntry
         * @property {number|Long|null} [id] MalwareLogEntry id
         * @property {number|Long|null} [scanTime] MalwareLogEntry scanTime
         * @property {string|null} [deviceSn] MalwareLogEntry deviceSn
         * @property {string|null} [deviceName] MalwareLogEntry deviceName
         * @property {string|null} [filePath] MalwareLogEntry filePath
         * @property {string|null} [scanResult] MalwareLogEntry scanResult
         * @property {string|null} [virusName] MalwareLogEntry virusName
         * @property {string|null} [virusDbVersion] MalwareLogEntry virusDbVersion
         * @property {string|null} [processResult] MalwareLogEntry processResult
         * @property {string|null} [failReason] MalwareLogEntry failReason
         * @property {string|null} [detail] MalwareLogEntry detail
         */

        /**
         * Constructs a new MalwareLogEntry.
         * @memberof usb_control
         * @classdesc Represents a MalwareLogEntry.
         * @implements IMalwareLogEntry
         * @constructor
         * @param {usb_control.IMalwareLogEntry=} [properties] Properties to set
         */
        function MalwareLogEntry(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * MalwareLogEntry id.
         * @member {number|Long} id
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.id = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * MalwareLogEntry scanTime.
         * @member {number|Long} scanTime
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.scanTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * MalwareLogEntry deviceSn.
         * @member {string} deviceSn
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.deviceSn = "";

        /**
         * MalwareLogEntry deviceName.
         * @member {string} deviceName
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.deviceName = "";

        /**
         * MalwareLogEntry filePath.
         * @member {string} filePath
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.filePath = "";

        /**
         * MalwareLogEntry scanResult.
         * @member {string} scanResult
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.scanResult = "";

        /**
         * MalwareLogEntry virusName.
         * @member {string} virusName
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.virusName = "";

        /**
         * MalwareLogEntry virusDbVersion.
         * @member {string} virusDbVersion
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.virusDbVersion = "";

        /**
         * MalwareLogEntry processResult.
         * @member {string} processResult
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.processResult = "";

        /**
         * MalwareLogEntry failReason.
         * @member {string} failReason
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.failReason = "";

        /**
         * MalwareLogEntry detail.
         * @member {string} detail
         * @memberof usb_control.MalwareLogEntry
         * @instance
         */
        MalwareLogEntry.prototype.detail = "";

        /**
         * Encodes the specified MalwareLogEntry message. Does not implicitly {@link usb_control.MalwareLogEntry.verify|verify} messages.
         * @function encode
         * @memberof usb_control.MalwareLogEntry
         * @static
         * @param {usb_control.IMalwareLogEntry} message MalwareLogEntry message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        MalwareLogEntry.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 0 =*/8).int64(message.id);
            if (message.scanTime != null && Object.hasOwnProperty.call(message, "scanTime"))
                writer.uint32(/* id 2, wireType 0 =*/16).int64(message.scanTime);
            if (message.deviceSn != null && Object.hasOwnProperty.call(message, "deviceSn"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.deviceSn);
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.deviceName);
            if (message.filePath != null && Object.hasOwnProperty.call(message, "filePath"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.filePath);
            if (message.scanResult != null && Object.hasOwnProperty.call(message, "scanResult"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.scanResult);
            if (message.virusName != null && Object.hasOwnProperty.call(message, "virusName"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.virusName);
            if (message.virusDbVersion != null && Object.hasOwnProperty.call(message, "virusDbVersion"))
                writer.uint32(/* id 8, wireType 2 =*/66).string(message.virusDbVersion);
            if (message.processResult != null && Object.hasOwnProperty.call(message, "processResult"))
                writer.uint32(/* id 9, wireType 2 =*/74).string(message.processResult);
            if (message.failReason != null && Object.hasOwnProperty.call(message, "failReason"))
                writer.uint32(/* id 10, wireType 2 =*/82).string(message.failReason);
            if (message.detail != null && Object.hasOwnProperty.call(message, "detail"))
                writer.uint32(/* id 11, wireType 2 =*/90).string(message.detail);
            return writer;
        };

        /**
         * Encodes the specified MalwareLogEntry message, length delimited. Does not implicitly {@link usb_control.MalwareLogEntry.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.MalwareLogEntry
         * @static
         * @param {usb_control.IMalwareLogEntry} message MalwareLogEntry message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        MalwareLogEntry.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a MalwareLogEntry message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.MalwareLogEntry
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.MalwareLogEntry} MalwareLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        MalwareLogEntry.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.MalwareLogEntry();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.id = reader.int64();
                        break;
                    }
                case 2: {
                        message.scanTime = reader.int64();
                        break;
                    }
                case 3: {
                        message.deviceSn = reader.string();
                        break;
                    }
                case 4: {
                        message.deviceName = reader.string();
                        break;
                    }
                case 5: {
                        message.filePath = reader.string();
                        break;
                    }
                case 6: {
                        message.scanResult = reader.string();
                        break;
                    }
                case 7: {
                        message.virusName = reader.string();
                        break;
                    }
                case 8: {
                        message.virusDbVersion = reader.string();
                        break;
                    }
                case 9: {
                        message.processResult = reader.string();
                        break;
                    }
                case 10: {
                        message.failReason = reader.string();
                        break;
                    }
                case 11: {
                        message.detail = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a MalwareLogEntry message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.MalwareLogEntry
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.MalwareLogEntry} MalwareLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        MalwareLogEntry.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a MalwareLogEntry message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.MalwareLogEntry
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.MalwareLogEntry} MalwareLogEntry
         */
        MalwareLogEntry.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.MalwareLogEntry)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.MalwareLogEntry: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.MalwareLogEntry();
            if (object.id != null)
                if ($util.Long)
                    message.id = $util.Long.fromValue(object.id, false);
                else if (typeof object.id === "string")
                    message.id = parseInt(object.id, 10);
                else if (typeof object.id === "number")
                    message.id = object.id;
                else if (typeof object.id === "object")
                    message.id = new $util.LongBits(object.id.low >>> 0, object.id.high >>> 0).toNumber();
            if (object.scanTime != null)
                if ($util.Long)
                    message.scanTime = $util.Long.fromValue(object.scanTime, false);
                else if (typeof object.scanTime === "string")
                    message.scanTime = parseInt(object.scanTime, 10);
                else if (typeof object.scanTime === "number")
                    message.scanTime = object.scanTime;
                else if (typeof object.scanTime === "object")
                    message.scanTime = new $util.LongBits(object.scanTime.low >>> 0, object.scanTime.high >>> 0).toNumber();
            if (object.deviceSn != null)
                message.deviceSn = String(object.deviceSn);
            if (object.deviceName != null)
                message.deviceName = String(object.deviceName);
            if (object.filePath != null)
                message.filePath = String(object.filePath);
            if (object.scanResult != null)
                message.scanResult = String(object.scanResult);
            if (object.virusName != null)
                message.virusName = String(object.virusName);
            if (object.virusDbVersion != null)
                message.virusDbVersion = String(object.virusDbVersion);
            if (object.processResult != null)
                message.processResult = String(object.processResult);
            if (object.failReason != null)
                message.failReason = String(object.failReason);
            if (object.detail != null)
                message.detail = String(object.detail);
            return message;
        };

        /**
         * Creates a plain object from a MalwareLogEntry message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.MalwareLogEntry
         * @static
         * @param {usb_control.MalwareLogEntry} message MalwareLogEntry
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        MalwareLogEntry.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.id = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.id = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.scanTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.scanTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.deviceSn = "";
                object.deviceName = "";
                object.filePath = "";
                object.scanResult = "";
                object.virusName = "";
                object.virusDbVersion = "";
                object.processResult = "";
                object.failReason = "";
                object.detail = "";
            }
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.id = typeof message.id === "number" ? BigInt(message.id) : $util.Long.fromBits(message.id.low >>> 0, message.id.high >>> 0, false).toBigInt();
                else if (typeof message.id === "number")
                    object.id = options.longs === String ? String(message.id) : message.id;
                else
                    object.id = options.longs === String ? $util.Long.prototype.toString.call(message.id) : options.longs === Number ? new $util.LongBits(message.id.low >>> 0, message.id.high >>> 0).toNumber() : message.id;
            if (message.scanTime != null && Object.hasOwnProperty.call(message, "scanTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.scanTime = typeof message.scanTime === "number" ? BigInt(message.scanTime) : $util.Long.fromBits(message.scanTime.low >>> 0, message.scanTime.high >>> 0, false).toBigInt();
                else if (typeof message.scanTime === "number")
                    object.scanTime = options.longs === String ? String(message.scanTime) : message.scanTime;
                else
                    object.scanTime = options.longs === String ? $util.Long.prototype.toString.call(message.scanTime) : options.longs === Number ? new $util.LongBits(message.scanTime.low >>> 0, message.scanTime.high >>> 0).toNumber() : message.scanTime;
            if (message.deviceSn != null && Object.hasOwnProperty.call(message, "deviceSn"))
                object.deviceSn = message.deviceSn;
            if (message.deviceName != null && Object.hasOwnProperty.call(message, "deviceName"))
                object.deviceName = message.deviceName;
            if (message.filePath != null && Object.hasOwnProperty.call(message, "filePath"))
                object.filePath = message.filePath;
            if (message.scanResult != null && Object.hasOwnProperty.call(message, "scanResult"))
                object.scanResult = message.scanResult;
            if (message.virusName != null && Object.hasOwnProperty.call(message, "virusName"))
                object.virusName = message.virusName;
            if (message.virusDbVersion != null && Object.hasOwnProperty.call(message, "virusDbVersion"))
                object.virusDbVersion = message.virusDbVersion;
            if (message.processResult != null && Object.hasOwnProperty.call(message, "processResult"))
                object.processResult = message.processResult;
            if (message.failReason != null && Object.hasOwnProperty.call(message, "failReason"))
                object.failReason = message.failReason;
            if (message.detail != null && Object.hasOwnProperty.call(message, "detail"))
                object.detail = message.detail;
            return object;
        };

        /**
         * Converts this MalwareLogEntry to JSON.
         * @function toJSON
         * @memberof usb_control.MalwareLogEntry
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        MalwareLogEntry.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for MalwareLogEntry
         * @function getTypeUrl
         * @memberof usb_control.MalwareLogEntry
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        MalwareLogEntry.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.MalwareLogEntry";
        };

        return MalwareLogEntry;
    })();

    usb_control.OperationLogEntry = (function() {

        /**
         * Properties of an OperationLogEntry.
         * @memberof usb_control
         * @interface IOperationLogEntry
         * @property {number|Long|null} [id] OperationLogEntry id
         * @property {number|Long|null} [opTime] OperationLogEntry opTime
         * @property {string|null} [username] OperationLogEntry username
         * @property {string|null} [role] OperationLogEntry role
         * @property {string|null} [logCategory] OperationLogEntry logCategory
         * @property {string|null} [actionType] OperationLogEntry actionType
         * @property {string|null} [target] OperationLogEntry target
         * @property {string|null} [relatedFile] OperationLogEntry relatedFile
         * @property {string|null} [relatedVersion] OperationLogEntry relatedVersion
         * @property {string|null} [result] OperationLogEntry result
         * @property {string|null} [failReason] OperationLogEntry failReason
         * @property {string|null} [detail] OperationLogEntry detail
         * @property {string|null} [sourceIp] OperationLogEntry sourceIp
         * @property {string|null} [appVersion] OperationLogEntry appVersion
         * @property {string|null} [sessionId] OperationLogEntry sessionId
         * @property {string|null} [requestId] OperationLogEntry requestId
         * @property {string|null} [beforeValue] OperationLogEntry beforeValue
         * @property {string|null} [afterValue] OperationLogEntry afterValue
         */

        /**
         * Constructs a new OperationLogEntry.
         * @memberof usb_control
         * @classdesc Represents an OperationLogEntry.
         * @implements IOperationLogEntry
         * @constructor
         * @param {usb_control.IOperationLogEntry=} [properties] Properties to set
         */
        function OperationLogEntry(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * OperationLogEntry id.
         * @member {number|Long} id
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.id = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * OperationLogEntry opTime.
         * @member {number|Long} opTime
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.opTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * OperationLogEntry username.
         * @member {string} username
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.username = "";

        /**
         * OperationLogEntry role.
         * @member {string} role
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.role = "";

        /**
         * OperationLogEntry logCategory.
         * @member {string} logCategory
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.logCategory = "";

        /**
         * OperationLogEntry actionType.
         * @member {string} actionType
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.actionType = "";

        /**
         * OperationLogEntry target.
         * @member {string} target
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.target = "";

        /**
         * OperationLogEntry relatedFile.
         * @member {string} relatedFile
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.relatedFile = "";

        /**
         * OperationLogEntry relatedVersion.
         * @member {string} relatedVersion
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.relatedVersion = "";

        /**
         * OperationLogEntry result.
         * @member {string} result
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.result = "";

        /**
         * OperationLogEntry failReason.
         * @member {string} failReason
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.failReason = "";

        /**
         * OperationLogEntry detail.
         * @member {string} detail
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.detail = "";

        /**
         * OperationLogEntry sourceIp.
         * @member {string} sourceIp
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.sourceIp = "";

        /**
         * OperationLogEntry appVersion.
         * @member {string} appVersion
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.appVersion = "";

        /**
         * OperationLogEntry sessionId.
         * @member {string} sessionId
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.sessionId = "";

        /**
         * OperationLogEntry requestId.
         * @member {string} requestId
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.requestId = "";

        /**
         * OperationLogEntry beforeValue.
         * @member {string} beforeValue
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.beforeValue = "";

        /**
         * OperationLogEntry afterValue.
         * @member {string} afterValue
         * @memberof usb_control.OperationLogEntry
         * @instance
         */
        OperationLogEntry.prototype.afterValue = "";

        /**
         * Encodes the specified OperationLogEntry message. Does not implicitly {@link usb_control.OperationLogEntry.verify|verify} messages.
         * @function encode
         * @memberof usb_control.OperationLogEntry
         * @static
         * @param {usb_control.IOperationLogEntry} message OperationLogEntry message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        OperationLogEntry.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 0 =*/8).int64(message.id);
            if (message.opTime != null && Object.hasOwnProperty.call(message, "opTime"))
                writer.uint32(/* id 2, wireType 0 =*/16).int64(message.opTime);
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.username);
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.role);
            if (message.logCategory != null && Object.hasOwnProperty.call(message, "logCategory"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.logCategory);
            if (message.actionType != null && Object.hasOwnProperty.call(message, "actionType"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.actionType);
            if (message.target != null && Object.hasOwnProperty.call(message, "target"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.target);
            if (message.relatedFile != null && Object.hasOwnProperty.call(message, "relatedFile"))
                writer.uint32(/* id 8, wireType 2 =*/66).string(message.relatedFile);
            if (message.relatedVersion != null && Object.hasOwnProperty.call(message, "relatedVersion"))
                writer.uint32(/* id 9, wireType 2 =*/74).string(message.relatedVersion);
            if (message.result != null && Object.hasOwnProperty.call(message, "result"))
                writer.uint32(/* id 10, wireType 2 =*/82).string(message.result);
            if (message.failReason != null && Object.hasOwnProperty.call(message, "failReason"))
                writer.uint32(/* id 11, wireType 2 =*/90).string(message.failReason);
            if (message.detail != null && Object.hasOwnProperty.call(message, "detail"))
                writer.uint32(/* id 12, wireType 2 =*/98).string(message.detail);
            if (message.sourceIp != null && Object.hasOwnProperty.call(message, "sourceIp"))
                writer.uint32(/* id 13, wireType 2 =*/106).string(message.sourceIp);
            if (message.appVersion != null && Object.hasOwnProperty.call(message, "appVersion"))
                writer.uint32(/* id 14, wireType 2 =*/114).string(message.appVersion);
            if (message.sessionId != null && Object.hasOwnProperty.call(message, "sessionId"))
                writer.uint32(/* id 15, wireType 2 =*/122).string(message.sessionId);
            if (message.requestId != null && Object.hasOwnProperty.call(message, "requestId"))
                writer.uint32(/* id 16, wireType 2 =*/130).string(message.requestId);
            if (message.beforeValue != null && Object.hasOwnProperty.call(message, "beforeValue"))
                writer.uint32(/* id 17, wireType 2 =*/138).string(message.beforeValue);
            if (message.afterValue != null && Object.hasOwnProperty.call(message, "afterValue"))
                writer.uint32(/* id 18, wireType 2 =*/146).string(message.afterValue);
            return writer;
        };

        /**
         * Encodes the specified OperationLogEntry message, length delimited. Does not implicitly {@link usb_control.OperationLogEntry.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.OperationLogEntry
         * @static
         * @param {usb_control.IOperationLogEntry} message OperationLogEntry message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        OperationLogEntry.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes an OperationLogEntry message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.OperationLogEntry
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.OperationLogEntry} OperationLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        OperationLogEntry.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.OperationLogEntry();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.id = reader.int64();
                        break;
                    }
                case 2: {
                        message.opTime = reader.int64();
                        break;
                    }
                case 3: {
                        message.username = reader.string();
                        break;
                    }
                case 4: {
                        message.role = reader.string();
                        break;
                    }
                case 5: {
                        message.logCategory = reader.string();
                        break;
                    }
                case 6: {
                        message.actionType = reader.string();
                        break;
                    }
                case 7: {
                        message.target = reader.string();
                        break;
                    }
                case 8: {
                        message.relatedFile = reader.string();
                        break;
                    }
                case 9: {
                        message.relatedVersion = reader.string();
                        break;
                    }
                case 10: {
                        message.result = reader.string();
                        break;
                    }
                case 11: {
                        message.failReason = reader.string();
                        break;
                    }
                case 12: {
                        message.detail = reader.string();
                        break;
                    }
                case 13: {
                        message.sourceIp = reader.string();
                        break;
                    }
                case 14: {
                        message.appVersion = reader.string();
                        break;
                    }
                case 15: {
                        message.sessionId = reader.string();
                        break;
                    }
                case 16: {
                        message.requestId = reader.string();
                        break;
                    }
                case 17: {
                        message.beforeValue = reader.string();
                        break;
                    }
                case 18: {
                        message.afterValue = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an OperationLogEntry message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.OperationLogEntry
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.OperationLogEntry} OperationLogEntry
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        OperationLogEntry.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates an OperationLogEntry message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.OperationLogEntry
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.OperationLogEntry} OperationLogEntry
         */
        OperationLogEntry.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.OperationLogEntry)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.OperationLogEntry: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.OperationLogEntry();
            if (object.id != null)
                if ($util.Long)
                    message.id = $util.Long.fromValue(object.id, false);
                else if (typeof object.id === "string")
                    message.id = parseInt(object.id, 10);
                else if (typeof object.id === "number")
                    message.id = object.id;
                else if (typeof object.id === "object")
                    message.id = new $util.LongBits(object.id.low >>> 0, object.id.high >>> 0).toNumber();
            if (object.opTime != null)
                if ($util.Long)
                    message.opTime = $util.Long.fromValue(object.opTime, false);
                else if (typeof object.opTime === "string")
                    message.opTime = parseInt(object.opTime, 10);
                else if (typeof object.opTime === "number")
                    message.opTime = object.opTime;
                else if (typeof object.opTime === "object")
                    message.opTime = new $util.LongBits(object.opTime.low >>> 0, object.opTime.high >>> 0).toNumber();
            if (object.username != null)
                message.username = String(object.username);
            if (object.role != null)
                message.role = String(object.role);
            if (object.logCategory != null)
                message.logCategory = String(object.logCategory);
            if (object.actionType != null)
                message.actionType = String(object.actionType);
            if (object.target != null)
                message.target = String(object.target);
            if (object.relatedFile != null)
                message.relatedFile = String(object.relatedFile);
            if (object.relatedVersion != null)
                message.relatedVersion = String(object.relatedVersion);
            if (object.result != null)
                message.result = String(object.result);
            if (object.failReason != null)
                message.failReason = String(object.failReason);
            if (object.detail != null)
                message.detail = String(object.detail);
            if (object.sourceIp != null)
                message.sourceIp = String(object.sourceIp);
            if (object.appVersion != null)
                message.appVersion = String(object.appVersion);
            if (object.sessionId != null)
                message.sessionId = String(object.sessionId);
            if (object.requestId != null)
                message.requestId = String(object.requestId);
            if (object.beforeValue != null)
                message.beforeValue = String(object.beforeValue);
            if (object.afterValue != null)
                message.afterValue = String(object.afterValue);
            return message;
        };

        /**
         * Creates a plain object from an OperationLogEntry message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.OperationLogEntry
         * @static
         * @param {usb_control.OperationLogEntry} message OperationLogEntry
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        OperationLogEntry.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.id = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.id = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.opTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.opTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.username = "";
                object.role = "";
                object.logCategory = "";
                object.actionType = "";
                object.target = "";
                object.relatedFile = "";
                object.relatedVersion = "";
                object.result = "";
                object.failReason = "";
                object.detail = "";
                object.sourceIp = "";
                object.appVersion = "";
                object.sessionId = "";
                object.requestId = "";
                object.beforeValue = "";
                object.afterValue = "";
            }
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.id = typeof message.id === "number" ? BigInt(message.id) : $util.Long.fromBits(message.id.low >>> 0, message.id.high >>> 0, false).toBigInt();
                else if (typeof message.id === "number")
                    object.id = options.longs === String ? String(message.id) : message.id;
                else
                    object.id = options.longs === String ? $util.Long.prototype.toString.call(message.id) : options.longs === Number ? new $util.LongBits(message.id.low >>> 0, message.id.high >>> 0).toNumber() : message.id;
            if (message.opTime != null && Object.hasOwnProperty.call(message, "opTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.opTime = typeof message.opTime === "number" ? BigInt(message.opTime) : $util.Long.fromBits(message.opTime.low >>> 0, message.opTime.high >>> 0, false).toBigInt();
                else if (typeof message.opTime === "number")
                    object.opTime = options.longs === String ? String(message.opTime) : message.opTime;
                else
                    object.opTime = options.longs === String ? $util.Long.prototype.toString.call(message.opTime) : options.longs === Number ? new $util.LongBits(message.opTime.low >>> 0, message.opTime.high >>> 0).toNumber() : message.opTime;
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                object.username = message.username;
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                object.role = message.role;
            if (message.logCategory != null && Object.hasOwnProperty.call(message, "logCategory"))
                object.logCategory = message.logCategory;
            if (message.actionType != null && Object.hasOwnProperty.call(message, "actionType"))
                object.actionType = message.actionType;
            if (message.target != null && Object.hasOwnProperty.call(message, "target"))
                object.target = message.target;
            if (message.relatedFile != null && Object.hasOwnProperty.call(message, "relatedFile"))
                object.relatedFile = message.relatedFile;
            if (message.relatedVersion != null && Object.hasOwnProperty.call(message, "relatedVersion"))
                object.relatedVersion = message.relatedVersion;
            if (message.result != null && Object.hasOwnProperty.call(message, "result"))
                object.result = message.result;
            if (message.failReason != null && Object.hasOwnProperty.call(message, "failReason"))
                object.failReason = message.failReason;
            if (message.detail != null && Object.hasOwnProperty.call(message, "detail"))
                object.detail = message.detail;
            if (message.sourceIp != null && Object.hasOwnProperty.call(message, "sourceIp"))
                object.sourceIp = message.sourceIp;
            if (message.appVersion != null && Object.hasOwnProperty.call(message, "appVersion"))
                object.appVersion = message.appVersion;
            if (message.sessionId != null && Object.hasOwnProperty.call(message, "sessionId"))
                object.sessionId = message.sessionId;
            if (message.requestId != null && Object.hasOwnProperty.call(message, "requestId"))
                object.requestId = message.requestId;
            if (message.beforeValue != null && Object.hasOwnProperty.call(message, "beforeValue"))
                object.beforeValue = message.beforeValue;
            if (message.afterValue != null && Object.hasOwnProperty.call(message, "afterValue"))
                object.afterValue = message.afterValue;
            return object;
        };

        /**
         * Converts this OperationLogEntry to JSON.
         * @function toJSON
         * @memberof usb_control.OperationLogEntry
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        OperationLogEntry.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for OperationLogEntry
         * @function getTypeUrl
         * @memberof usb_control.OperationLogEntry
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        OperationLogEntry.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.OperationLogEntry";
        };

        return OperationLogEntry;
    })();

    usb_control.RspQueryLogs = (function() {

        /**
         * Properties of a RspQueryLogs.
         * @memberof usb_control
         * @interface IRspQueryLogs
         * @property {boolean|null} [success] RspQueryLogs success
         * @property {Array.<usb_control.IUsbAuditLogEntry>|null} [usbAuditEntries] RspQueryLogs usbAuditEntries
         * @property {Array.<usb_control.IMalwareLogEntry>|null} [malwareEntries] RspQueryLogs malwareEntries
         * @property {Array.<usb_control.IOperationLogEntry>|null} [operationEntries] RspQueryLogs operationEntries
         * @property {number|null} [total] RspQueryLogs total
         * @property {number|null} [page] RspQueryLogs page
         * @property {number|null} [pageSize] RspQueryLogs pageSize
         * @property {number|null} [resultCode] RspQueryLogs resultCode
         * @property {string|null} [errorMessage] RspQueryLogs errorMessage
         */

        /**
         * Constructs a new RspQueryLogs.
         * @memberof usb_control
         * @classdesc Represents a RspQueryLogs.
         * @implements IRspQueryLogs
         * @constructor
         * @param {usb_control.IRspQueryLogs=} [properties] Properties to set
         */
        function RspQueryLogs(properties) {
            this.usbAuditEntries = [];
            this.malwareEntries = [];
            this.operationEntries = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspQueryLogs success.
         * @member {boolean} success
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.success = false;

        /**
         * RspQueryLogs usbAuditEntries.
         * @member {Array.<usb_control.IUsbAuditLogEntry>} usbAuditEntries
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.usbAuditEntries = $util.emptyArray;

        /**
         * RspQueryLogs malwareEntries.
         * @member {Array.<usb_control.IMalwareLogEntry>} malwareEntries
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.malwareEntries = $util.emptyArray;

        /**
         * RspQueryLogs operationEntries.
         * @member {Array.<usb_control.IOperationLogEntry>} operationEntries
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.operationEntries = $util.emptyArray;

        /**
         * RspQueryLogs total.
         * @member {number} total
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.total = 0;

        /**
         * RspQueryLogs page.
         * @member {number} page
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.page = 0;

        /**
         * RspQueryLogs pageSize.
         * @member {number} pageSize
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.pageSize = 0;

        /**
         * RspQueryLogs resultCode.
         * @member {number} resultCode
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.resultCode = 0;

        /**
         * RspQueryLogs errorMessage.
         * @member {string} errorMessage
         * @memberof usb_control.RspQueryLogs
         * @instance
         */
        RspQueryLogs.prototype.errorMessage = "";

        /**
         * Encodes the specified RspQueryLogs message. Does not implicitly {@link usb_control.RspQueryLogs.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspQueryLogs
         * @static
         * @param {usb_control.IRspQueryLogs} message RspQueryLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspQueryLogs.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
            if (message.usbAuditEntries != null && message.usbAuditEntries.length)
                for (let i = 0; i < message.usbAuditEntries.length; ++i)
                    $root.usb_control.UsbAuditLogEntry.encode(message.usbAuditEntries[i], writer.uint32(/* id 2, wireType 2 =*/18).fork(), q + 1).ldelim();
            if (message.malwareEntries != null && message.malwareEntries.length)
                for (let i = 0; i < message.malwareEntries.length; ++i)
                    $root.usb_control.MalwareLogEntry.encode(message.malwareEntries[i], writer.uint32(/* id 3, wireType 2 =*/26).fork(), q + 1).ldelim();
            if (message.operationEntries != null && message.operationEntries.length)
                for (let i = 0; i < message.operationEntries.length; ++i)
                    $root.usb_control.OperationLogEntry.encode(message.operationEntries[i], writer.uint32(/* id 4, wireType 2 =*/34).fork(), q + 1).ldelim();
            if (message.total != null && Object.hasOwnProperty.call(message, "total"))
                writer.uint32(/* id 5, wireType 0 =*/40).int32(message.total);
            if (message.page != null && Object.hasOwnProperty.call(message, "page"))
                writer.uint32(/* id 6, wireType 0 =*/48).int32(message.page);
            if (message.pageSize != null && Object.hasOwnProperty.call(message, "pageSize"))
                writer.uint32(/* id 7, wireType 0 =*/56).int32(message.pageSize);
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                writer.uint32(/* id 8, wireType 0 =*/64).int32(message.resultCode);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 9, wireType 2 =*/74).string(message.errorMessage);
            return writer;
        };

        /**
         * Encodes the specified RspQueryLogs message, length delimited. Does not implicitly {@link usb_control.RspQueryLogs.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspQueryLogs
         * @static
         * @param {usb_control.IRspQueryLogs} message RspQueryLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspQueryLogs.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspQueryLogs message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspQueryLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspQueryLogs} RspQueryLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspQueryLogs.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspQueryLogs();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.success = reader.bool();
                        break;
                    }
                case 2: {
                        if (!(message.usbAuditEntries && message.usbAuditEntries.length))
                            message.usbAuditEntries = [];
                        message.usbAuditEntries.push($root.usb_control.UsbAuditLogEntry.decode(reader, reader.uint32(), undefined, long + 1));
                        break;
                    }
                case 3: {
                        if (!(message.malwareEntries && message.malwareEntries.length))
                            message.malwareEntries = [];
                        message.malwareEntries.push($root.usb_control.MalwareLogEntry.decode(reader, reader.uint32(), undefined, long + 1));
                        break;
                    }
                case 4: {
                        if (!(message.operationEntries && message.operationEntries.length))
                            message.operationEntries = [];
                        message.operationEntries.push($root.usb_control.OperationLogEntry.decode(reader, reader.uint32(), undefined, long + 1));
                        break;
                    }
                case 5: {
                        message.total = reader.int32();
                        break;
                    }
                case 6: {
                        message.page = reader.int32();
                        break;
                    }
                case 7: {
                        message.pageSize = reader.int32();
                        break;
                    }
                case 8: {
                        message.resultCode = reader.int32();
                        break;
                    }
                case 9: {
                        message.errorMessage = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspQueryLogs message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspQueryLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspQueryLogs} RspQueryLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspQueryLogs.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspQueryLogs message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspQueryLogs
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspQueryLogs} RspQueryLogs
         */
        RspQueryLogs.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspQueryLogs)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspQueryLogs: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspQueryLogs();
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.usbAuditEntries) {
                if (!Array.isArray(object.usbAuditEntries))
                    throw TypeError(".usb_control.RspQueryLogs.usbAuditEntries: array expected");
                message.usbAuditEntries = [];
                for (let i = 0; i < object.usbAuditEntries.length; ++i) {
                    if (!$util.isObject(object.usbAuditEntries[i]))
                        throw TypeError(".usb_control.RspQueryLogs.usbAuditEntries: object expected");
                    message.usbAuditEntries[i] = $root.usb_control.UsbAuditLogEntry.fromObject(object.usbAuditEntries[i], long + 1);
                }
            }
            if (object.malwareEntries) {
                if (!Array.isArray(object.malwareEntries))
                    throw TypeError(".usb_control.RspQueryLogs.malwareEntries: array expected");
                message.malwareEntries = [];
                for (let i = 0; i < object.malwareEntries.length; ++i) {
                    if (!$util.isObject(object.malwareEntries[i]))
                        throw TypeError(".usb_control.RspQueryLogs.malwareEntries: object expected");
                    message.malwareEntries[i] = $root.usb_control.MalwareLogEntry.fromObject(object.malwareEntries[i], long + 1);
                }
            }
            if (object.operationEntries) {
                if (!Array.isArray(object.operationEntries))
                    throw TypeError(".usb_control.RspQueryLogs.operationEntries: array expected");
                message.operationEntries = [];
                for (let i = 0; i < object.operationEntries.length; ++i) {
                    if (!$util.isObject(object.operationEntries[i]))
                        throw TypeError(".usb_control.RspQueryLogs.operationEntries: object expected");
                    message.operationEntries[i] = $root.usb_control.OperationLogEntry.fromObject(object.operationEntries[i], long + 1);
                }
            }
            if (object.total != null)
                message.total = object.total | 0;
            if (object.page != null)
                message.page = object.page | 0;
            if (object.pageSize != null)
                message.pageSize = object.pageSize | 0;
            if (object.resultCode != null)
                message.resultCode = object.resultCode | 0;
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            return message;
        };

        /**
         * Creates a plain object from a RspQueryLogs message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspQueryLogs
         * @static
         * @param {usb_control.RspQueryLogs} message RspQueryLogs
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspQueryLogs.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.arrays || options.defaults) {
                object.usbAuditEntries = [];
                object.malwareEntries = [];
                object.operationEntries = [];
            }
            if (options.defaults) {
                object.success = false;
                object.total = 0;
                object.page = 0;
                object.pageSize = 0;
                object.resultCode = 0;
                object.errorMessage = "";
            }
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                object.success = message.success;
            if (message.usbAuditEntries && message.usbAuditEntries.length) {
                object.usbAuditEntries = [];
                for (let j = 0; j < message.usbAuditEntries.length; ++j)
                    object.usbAuditEntries[j] = $root.usb_control.UsbAuditLogEntry.toObject(message.usbAuditEntries[j], options, q + 1);
            }
            if (message.malwareEntries && message.malwareEntries.length) {
                object.malwareEntries = [];
                for (let j = 0; j < message.malwareEntries.length; ++j)
                    object.malwareEntries[j] = $root.usb_control.MalwareLogEntry.toObject(message.malwareEntries[j], options, q + 1);
            }
            if (message.operationEntries && message.operationEntries.length) {
                object.operationEntries = [];
                for (let j = 0; j < message.operationEntries.length; ++j)
                    object.operationEntries[j] = $root.usb_control.OperationLogEntry.toObject(message.operationEntries[j], options, q + 1);
            }
            if (message.total != null && Object.hasOwnProperty.call(message, "total"))
                object.total = message.total;
            if (message.page != null && Object.hasOwnProperty.call(message, "page"))
                object.page = message.page;
            if (message.pageSize != null && Object.hasOwnProperty.call(message, "pageSize"))
                object.pageSize = message.pageSize;
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                object.resultCode = message.resultCode;
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                object.errorMessage = message.errorMessage;
            return object;
        };

        /**
         * Converts this RspQueryLogs to JSON.
         * @function toJSON
         * @memberof usb_control.RspQueryLogs
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspQueryLogs.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspQueryLogs
         * @function getTypeUrl
         * @memberof usb_control.RspQueryLogs
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspQueryLogs.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspQueryLogs";
        };

        return RspQueryLogs;
    })();

    usb_control.CmdExportLogs = (function() {

        /**
         * Properties of a CmdExportLogs.
         * @memberof usb_control
         * @interface ICmdExportLogs
         * @property {string|null} [sessionToken] CmdExportLogs sessionToken
         * @property {string|null} [logType] CmdExportLogs logType
         * @property {number|Long|null} [startTime] CmdExportLogs startTime
         * @property {number|Long|null} [endTime] CmdExportLogs endTime
         * @property {string|null} [keyword] CmdExportLogs keyword
         * @property {string|null} [eventType] CmdExportLogs eventType
         */

        /**
         * Constructs a new CmdExportLogs.
         * @memberof usb_control
         * @classdesc Represents a CmdExportLogs.
         * @implements ICmdExportLogs
         * @constructor
         * @param {usb_control.ICmdExportLogs=} [properties] Properties to set
         */
        function CmdExportLogs(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdExportLogs sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdExportLogs
         * @instance
         */
        CmdExportLogs.prototype.sessionToken = "";

        /**
         * CmdExportLogs logType.
         * @member {string} logType
         * @memberof usb_control.CmdExportLogs
         * @instance
         */
        CmdExportLogs.prototype.logType = "";

        /**
         * CmdExportLogs startTime.
         * @member {number|Long} startTime
         * @memberof usb_control.CmdExportLogs
         * @instance
         */
        CmdExportLogs.prototype.startTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * CmdExportLogs endTime.
         * @member {number|Long} endTime
         * @memberof usb_control.CmdExportLogs
         * @instance
         */
        CmdExportLogs.prototype.endTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * CmdExportLogs keyword.
         * @member {string} keyword
         * @memberof usb_control.CmdExportLogs
         * @instance
         */
        CmdExportLogs.prototype.keyword = "";

        /**
         * CmdExportLogs eventType.
         * @member {string} eventType
         * @memberof usb_control.CmdExportLogs
         * @instance
         */
        CmdExportLogs.prototype.eventType = "";

        /**
         * Encodes the specified CmdExportLogs message. Does not implicitly {@link usb_control.CmdExportLogs.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdExportLogs
         * @static
         * @param {usb_control.ICmdExportLogs} message CmdExportLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdExportLogs.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.logType != null && Object.hasOwnProperty.call(message, "logType"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.logType);
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                writer.uint32(/* id 3, wireType 0 =*/24).int64(message.startTime);
            if (message.endTime != null && Object.hasOwnProperty.call(message, "endTime"))
                writer.uint32(/* id 4, wireType 0 =*/32).int64(message.endTime);
            if (message.keyword != null && Object.hasOwnProperty.call(message, "keyword"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.keyword);
            if (message.eventType != null && Object.hasOwnProperty.call(message, "eventType"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.eventType);
            return writer;
        };

        /**
         * Encodes the specified CmdExportLogs message, length delimited. Does not implicitly {@link usb_control.CmdExportLogs.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdExportLogs
         * @static
         * @param {usb_control.ICmdExportLogs} message CmdExportLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdExportLogs.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdExportLogs message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdExportLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdExportLogs} CmdExportLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdExportLogs.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdExportLogs();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.logType = reader.string();
                        break;
                    }
                case 3: {
                        message.startTime = reader.int64();
                        break;
                    }
                case 4: {
                        message.endTime = reader.int64();
                        break;
                    }
                case 5: {
                        message.keyword = reader.string();
                        break;
                    }
                case 6: {
                        message.eventType = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdExportLogs message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdExportLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdExportLogs} CmdExportLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdExportLogs.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdExportLogs message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdExportLogs
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdExportLogs} CmdExportLogs
         */
        CmdExportLogs.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdExportLogs)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdExportLogs: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdExportLogs();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.logType != null)
                message.logType = String(object.logType);
            if (object.startTime != null)
                if ($util.Long)
                    message.startTime = $util.Long.fromValue(object.startTime, false);
                else if (typeof object.startTime === "string")
                    message.startTime = parseInt(object.startTime, 10);
                else if (typeof object.startTime === "number")
                    message.startTime = object.startTime;
                else if (typeof object.startTime === "object")
                    message.startTime = new $util.LongBits(object.startTime.low >>> 0, object.startTime.high >>> 0).toNumber();
            if (object.endTime != null)
                if ($util.Long)
                    message.endTime = $util.Long.fromValue(object.endTime, false);
                else if (typeof object.endTime === "string")
                    message.endTime = parseInt(object.endTime, 10);
                else if (typeof object.endTime === "number")
                    message.endTime = object.endTime;
                else if (typeof object.endTime === "object")
                    message.endTime = new $util.LongBits(object.endTime.low >>> 0, object.endTime.high >>> 0).toNumber();
            if (object.keyword != null)
                message.keyword = String(object.keyword);
            if (object.eventType != null)
                message.eventType = String(object.eventType);
            return message;
        };

        /**
         * Creates a plain object from a CmdExportLogs message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdExportLogs
         * @static
         * @param {usb_control.CmdExportLogs} message CmdExportLogs
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdExportLogs.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.logType = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.startTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.startTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.endTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.endTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.keyword = "";
                object.eventType = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.logType != null && Object.hasOwnProperty.call(message, "logType"))
                object.logType = message.logType;
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.startTime = typeof message.startTime === "number" ? BigInt(message.startTime) : $util.Long.fromBits(message.startTime.low >>> 0, message.startTime.high >>> 0, false).toBigInt();
                else if (typeof message.startTime === "number")
                    object.startTime = options.longs === String ? String(message.startTime) : message.startTime;
                else
                    object.startTime = options.longs === String ? $util.Long.prototype.toString.call(message.startTime) : options.longs === Number ? new $util.LongBits(message.startTime.low >>> 0, message.startTime.high >>> 0).toNumber() : message.startTime;
            if (message.endTime != null && Object.hasOwnProperty.call(message, "endTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.endTime = typeof message.endTime === "number" ? BigInt(message.endTime) : $util.Long.fromBits(message.endTime.low >>> 0, message.endTime.high >>> 0, false).toBigInt();
                else if (typeof message.endTime === "number")
                    object.endTime = options.longs === String ? String(message.endTime) : message.endTime;
                else
                    object.endTime = options.longs === String ? $util.Long.prototype.toString.call(message.endTime) : options.longs === Number ? new $util.LongBits(message.endTime.low >>> 0, message.endTime.high >>> 0).toNumber() : message.endTime;
            if (message.keyword != null && Object.hasOwnProperty.call(message, "keyword"))
                object.keyword = message.keyword;
            if (message.eventType != null && Object.hasOwnProperty.call(message, "eventType"))
                object.eventType = message.eventType;
            return object;
        };

        /**
         * Converts this CmdExportLogs to JSON.
         * @function toJSON
         * @memberof usb_control.CmdExportLogs
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdExportLogs.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdExportLogs
         * @function getTypeUrl
         * @memberof usb_control.CmdExportLogs
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdExportLogs.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdExportLogs";
        };

        return CmdExportLogs;
    })();

    usb_control.RspExportLogs = (function() {

        /**
         * Properties of a RspExportLogs.
         * @memberof usb_control
         * @interface IRspExportLogs
         * @property {boolean|null} [success] RspExportLogs success
         * @property {Uint8Array|null} [zipData] RspExportLogs zipData
         * @property {string|null} [suggestedFilename] RspExportLogs suggestedFilename
         * @property {number|null} [resultCode] RspExportLogs resultCode
         * @property {string|null} [errorMessage] RspExportLogs errorMessage
         */

        /**
         * Constructs a new RspExportLogs.
         * @memberof usb_control
         * @classdesc Represents a RspExportLogs.
         * @implements IRspExportLogs
         * @constructor
         * @param {usb_control.IRspExportLogs=} [properties] Properties to set
         */
        function RspExportLogs(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspExportLogs success.
         * @member {boolean} success
         * @memberof usb_control.RspExportLogs
         * @instance
         */
        RspExportLogs.prototype.success = false;

        /**
         * RspExportLogs zipData.
         * @member {Uint8Array} zipData
         * @memberof usb_control.RspExportLogs
         * @instance
         */
        RspExportLogs.prototype.zipData = $util.newBuffer([]);

        /**
         * RspExportLogs suggestedFilename.
         * @member {string} suggestedFilename
         * @memberof usb_control.RspExportLogs
         * @instance
         */
        RspExportLogs.prototype.suggestedFilename = "";

        /**
         * RspExportLogs resultCode.
         * @member {number} resultCode
         * @memberof usb_control.RspExportLogs
         * @instance
         */
        RspExportLogs.prototype.resultCode = 0;

        /**
         * RspExportLogs errorMessage.
         * @member {string} errorMessage
         * @memberof usb_control.RspExportLogs
         * @instance
         */
        RspExportLogs.prototype.errorMessage = "";

        /**
         * Encodes the specified RspExportLogs message. Does not implicitly {@link usb_control.RspExportLogs.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspExportLogs
         * @static
         * @param {usb_control.IRspExportLogs} message RspExportLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspExportLogs.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
            if (message.zipData != null && Object.hasOwnProperty.call(message, "zipData"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.zipData);
            if (message.suggestedFilename != null && Object.hasOwnProperty.call(message, "suggestedFilename"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.suggestedFilename);
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                writer.uint32(/* id 4, wireType 0 =*/32).int32(message.resultCode);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.errorMessage);
            return writer;
        };

        /**
         * Encodes the specified RspExportLogs message, length delimited. Does not implicitly {@link usb_control.RspExportLogs.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspExportLogs
         * @static
         * @param {usb_control.IRspExportLogs} message RspExportLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspExportLogs.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspExportLogs message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspExportLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspExportLogs} RspExportLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspExportLogs.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspExportLogs();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.success = reader.bool();
                        break;
                    }
                case 2: {
                        message.zipData = reader.bytes();
                        break;
                    }
                case 3: {
                        message.suggestedFilename = reader.string();
                        break;
                    }
                case 4: {
                        message.resultCode = reader.int32();
                        break;
                    }
                case 5: {
                        message.errorMessage = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspExportLogs message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspExportLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspExportLogs} RspExportLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspExportLogs.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspExportLogs message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspExportLogs
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspExportLogs} RspExportLogs
         */
        RspExportLogs.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspExportLogs)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspExportLogs: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspExportLogs();
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.zipData != null)
                if (typeof object.zipData === "string")
                    $util.base64.decode(object.zipData, message.zipData = $util.newBuffer($util.base64.length(object.zipData)), 0);
                else if (object.zipData.length >= 0)
                    message.zipData = object.zipData;
            if (object.suggestedFilename != null)
                message.suggestedFilename = String(object.suggestedFilename);
            if (object.resultCode != null)
                message.resultCode = object.resultCode | 0;
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            return message;
        };

        /**
         * Creates a plain object from a RspExportLogs message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspExportLogs
         * @static
         * @param {usb_control.RspExportLogs} message RspExportLogs
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspExportLogs.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.success = false;
                if (options.bytes === String)
                    object.zipData = "";
                else {
                    object.zipData = [];
                    if (options.bytes !== Array)
                        object.zipData = $util.newBuffer(object.zipData);
                }
                object.suggestedFilename = "";
                object.resultCode = 0;
                object.errorMessage = "";
            }
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                object.success = message.success;
            if (message.zipData != null && Object.hasOwnProperty.call(message, "zipData"))
                object.zipData = options.bytes === String ? $util.base64.encode(message.zipData, 0, message.zipData.length) : options.bytes === Array ? Array.prototype.slice.call(message.zipData) : message.zipData;
            if (message.suggestedFilename != null && Object.hasOwnProperty.call(message, "suggestedFilename"))
                object.suggestedFilename = message.suggestedFilename;
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                object.resultCode = message.resultCode;
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                object.errorMessage = message.errorMessage;
            return object;
        };

        /**
         * Converts this RspExportLogs to JSON.
         * @function toJSON
         * @memberof usb_control.RspExportLogs
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspExportLogs.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspExportLogs
         * @function getTypeUrl
         * @memberof usb_control.RspExportLogs
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspExportLogs.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspExportLogs";
        };

        return RspExportLogs;
    })();

    usb_control.CmdDeleteLogs = (function() {

        /**
         * Properties of a CmdDeleteLogs.
         * @memberof usb_control
         * @interface ICmdDeleteLogs
         * @property {string|null} [sessionToken] CmdDeleteLogs sessionToken
         * @property {string|null} [logType] CmdDeleteLogs logType
         * @property {number|Long|null} [startTime] CmdDeleteLogs startTime
         * @property {number|Long|null} [endTime] CmdDeleteLogs endTime
         */

        /**
         * Constructs a new CmdDeleteLogs.
         * @memberof usb_control
         * @classdesc Represents a CmdDeleteLogs.
         * @implements ICmdDeleteLogs
         * @constructor
         * @param {usb_control.ICmdDeleteLogs=} [properties] Properties to set
         */
        function CmdDeleteLogs(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdDeleteLogs sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdDeleteLogs
         * @instance
         */
        CmdDeleteLogs.prototype.sessionToken = "";

        /**
         * CmdDeleteLogs logType.
         * @member {string} logType
         * @memberof usb_control.CmdDeleteLogs
         * @instance
         */
        CmdDeleteLogs.prototype.logType = "";

        /**
         * CmdDeleteLogs startTime.
         * @member {number|Long} startTime
         * @memberof usb_control.CmdDeleteLogs
         * @instance
         */
        CmdDeleteLogs.prototype.startTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * CmdDeleteLogs endTime.
         * @member {number|Long} endTime
         * @memberof usb_control.CmdDeleteLogs
         * @instance
         */
        CmdDeleteLogs.prototype.endTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * Encodes the specified CmdDeleteLogs message. Does not implicitly {@link usb_control.CmdDeleteLogs.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdDeleteLogs
         * @static
         * @param {usb_control.ICmdDeleteLogs} message CmdDeleteLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdDeleteLogs.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.logType != null && Object.hasOwnProperty.call(message, "logType"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.logType);
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                writer.uint32(/* id 3, wireType 0 =*/24).int64(message.startTime);
            if (message.endTime != null && Object.hasOwnProperty.call(message, "endTime"))
                writer.uint32(/* id 4, wireType 0 =*/32).int64(message.endTime);
            return writer;
        };

        /**
         * Encodes the specified CmdDeleteLogs message, length delimited. Does not implicitly {@link usb_control.CmdDeleteLogs.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdDeleteLogs
         * @static
         * @param {usb_control.ICmdDeleteLogs} message CmdDeleteLogs message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdDeleteLogs.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdDeleteLogs message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdDeleteLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdDeleteLogs} CmdDeleteLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdDeleteLogs.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdDeleteLogs();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.logType = reader.string();
                        break;
                    }
                case 3: {
                        message.startTime = reader.int64();
                        break;
                    }
                case 4: {
                        message.endTime = reader.int64();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdDeleteLogs message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdDeleteLogs
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdDeleteLogs} CmdDeleteLogs
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdDeleteLogs.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdDeleteLogs message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdDeleteLogs
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdDeleteLogs} CmdDeleteLogs
         */
        CmdDeleteLogs.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdDeleteLogs)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdDeleteLogs: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdDeleteLogs();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.logType != null)
                message.logType = String(object.logType);
            if (object.startTime != null)
                if ($util.Long)
                    message.startTime = $util.Long.fromValue(object.startTime, false);
                else if (typeof object.startTime === "string")
                    message.startTime = parseInt(object.startTime, 10);
                else if (typeof object.startTime === "number")
                    message.startTime = object.startTime;
                else if (typeof object.startTime === "object")
                    message.startTime = new $util.LongBits(object.startTime.low >>> 0, object.startTime.high >>> 0).toNumber();
            if (object.endTime != null)
                if ($util.Long)
                    message.endTime = $util.Long.fromValue(object.endTime, false);
                else if (typeof object.endTime === "string")
                    message.endTime = parseInt(object.endTime, 10);
                else if (typeof object.endTime === "number")
                    message.endTime = object.endTime;
                else if (typeof object.endTime === "object")
                    message.endTime = new $util.LongBits(object.endTime.low >>> 0, object.endTime.high >>> 0).toNumber();
            return message;
        };

        /**
         * Creates a plain object from a CmdDeleteLogs message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdDeleteLogs
         * @static
         * @param {usb_control.CmdDeleteLogs} message CmdDeleteLogs
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdDeleteLogs.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.logType = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.startTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.startTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.endTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.endTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.logType != null && Object.hasOwnProperty.call(message, "logType"))
                object.logType = message.logType;
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.startTime = typeof message.startTime === "number" ? BigInt(message.startTime) : $util.Long.fromBits(message.startTime.low >>> 0, message.startTime.high >>> 0, false).toBigInt();
                else if (typeof message.startTime === "number")
                    object.startTime = options.longs === String ? String(message.startTime) : message.startTime;
                else
                    object.startTime = options.longs === String ? $util.Long.prototype.toString.call(message.startTime) : options.longs === Number ? new $util.LongBits(message.startTime.low >>> 0, message.startTime.high >>> 0).toNumber() : message.startTime;
            if (message.endTime != null && Object.hasOwnProperty.call(message, "endTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.endTime = typeof message.endTime === "number" ? BigInt(message.endTime) : $util.Long.fromBits(message.endTime.low >>> 0, message.endTime.high >>> 0, false).toBigInt();
                else if (typeof message.endTime === "number")
                    object.endTime = options.longs === String ? String(message.endTime) : message.endTime;
                else
                    object.endTime = options.longs === String ? $util.Long.prototype.toString.call(message.endTime) : options.longs === Number ? new $util.LongBits(message.endTime.low >>> 0, message.endTime.high >>> 0).toNumber() : message.endTime;
            return object;
        };

        /**
         * Converts this CmdDeleteLogs to JSON.
         * @function toJSON
         * @memberof usb_control.CmdDeleteLogs
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdDeleteLogs.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdDeleteLogs
         * @function getTypeUrl
         * @memberof usb_control.CmdDeleteLogs
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdDeleteLogs.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdDeleteLogs";
        };

        return CmdDeleteLogs;
    })();

    usb_control.CmdGetSystemInfo = (function() {

        /**
         * Properties of a CmdGetSystemInfo.
         * @memberof usb_control
         * @interface ICmdGetSystemInfo
         * @property {string|null} [sessionToken] CmdGetSystemInfo sessionToken
         */

        /**
         * Constructs a new CmdGetSystemInfo.
         * @memberof usb_control
         * @classdesc Represents a CmdGetSystemInfo.
         * @implements ICmdGetSystemInfo
         * @constructor
         * @param {usb_control.ICmdGetSystemInfo=} [properties] Properties to set
         */
        function CmdGetSystemInfo(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdGetSystemInfo sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdGetSystemInfo
         * @instance
         */
        CmdGetSystemInfo.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdGetSystemInfo message. Does not implicitly {@link usb_control.CmdGetSystemInfo.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdGetSystemInfo
         * @static
         * @param {usb_control.ICmdGetSystemInfo} message CmdGetSystemInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdGetSystemInfo.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdGetSystemInfo message, length delimited. Does not implicitly {@link usb_control.CmdGetSystemInfo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdGetSystemInfo
         * @static
         * @param {usb_control.ICmdGetSystemInfo} message CmdGetSystemInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdGetSystemInfo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdGetSystemInfo message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdGetSystemInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdGetSystemInfo} CmdGetSystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdGetSystemInfo.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdGetSystemInfo();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdGetSystemInfo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdGetSystemInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdGetSystemInfo} CmdGetSystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdGetSystemInfo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdGetSystemInfo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdGetSystemInfo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdGetSystemInfo} CmdGetSystemInfo
         */
        CmdGetSystemInfo.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdGetSystemInfo)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdGetSystemInfo: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdGetSystemInfo();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdGetSystemInfo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdGetSystemInfo
         * @static
         * @param {usb_control.CmdGetSystemInfo} message CmdGetSystemInfo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdGetSystemInfo.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdGetSystemInfo to JSON.
         * @function toJSON
         * @memberof usb_control.CmdGetSystemInfo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdGetSystemInfo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdGetSystemInfo
         * @function getTypeUrl
         * @memberof usb_control.CmdGetSystemInfo
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdGetSystemInfo.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdGetSystemInfo";
        };

        return CmdGetSystemInfo;
    })();

    usb_control.RspSystemInfo = (function() {

        /**
         * Properties of a RspSystemInfo.
         * @memberof usb_control
         * @interface IRspSystemInfo
         * @property {string|null} [systemVersion] RspSystemInfo systemVersion
         * @property {string|null} [virusDbVersion] RspSystemInfo virusDbVersion
         * @property {boolean|null} [authorized] RspSystemInfo authorized
         * @property {number|Long|null} [authExpireTime] RspSystemInfo authExpireTime
         * @property {string|null} [deviceDescription] RspSystemInfo deviceDescription
         * @property {number|Long|null} [virusDbUpdatedAt] RspSystemInfo virusDbUpdatedAt
         * @property {string|null} [authStatus] RspSystemInfo authStatus
         */

        /**
         * Constructs a new RspSystemInfo.
         * @memberof usb_control
         * @classdesc Represents a RspSystemInfo.
         * @implements IRspSystemInfo
         * @constructor
         * @param {usb_control.IRspSystemInfo=} [properties] Properties to set
         */
        function RspSystemInfo(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspSystemInfo systemVersion.
         * @member {string} systemVersion
         * @memberof usb_control.RspSystemInfo
         * @instance
         */
        RspSystemInfo.prototype.systemVersion = "";

        /**
         * RspSystemInfo virusDbVersion.
         * @member {string} virusDbVersion
         * @memberof usb_control.RspSystemInfo
         * @instance
         */
        RspSystemInfo.prototype.virusDbVersion = "";

        /**
         * RspSystemInfo authorized.
         * @member {boolean} authorized
         * @memberof usb_control.RspSystemInfo
         * @instance
         */
        RspSystemInfo.prototype.authorized = false;

        /**
         * RspSystemInfo authExpireTime.
         * @member {number|Long} authExpireTime
         * @memberof usb_control.RspSystemInfo
         * @instance
         */
        RspSystemInfo.prototype.authExpireTime = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * RspSystemInfo deviceDescription.
         * @member {string} deviceDescription
         * @memberof usb_control.RspSystemInfo
         * @instance
         */
        RspSystemInfo.prototype.deviceDescription = "";

        /**
         * RspSystemInfo virusDbUpdatedAt.
         * @member {number|Long} virusDbUpdatedAt
         * @memberof usb_control.RspSystemInfo
         * @instance
         */
        RspSystemInfo.prototype.virusDbUpdatedAt = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * RspSystemInfo authStatus.
         * @member {string} authStatus
         * @memberof usb_control.RspSystemInfo
         * @instance
         */
        RspSystemInfo.prototype.authStatus = "";

        /**
         * Encodes the specified RspSystemInfo message. Does not implicitly {@link usb_control.RspSystemInfo.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspSystemInfo
         * @static
         * @param {usb_control.IRspSystemInfo} message RspSystemInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspSystemInfo.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.systemVersion != null && Object.hasOwnProperty.call(message, "systemVersion"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.systemVersion);
            if (message.virusDbVersion != null && Object.hasOwnProperty.call(message, "virusDbVersion"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.virusDbVersion);
            if (message.authorized != null && Object.hasOwnProperty.call(message, "authorized"))
                writer.uint32(/* id 3, wireType 0 =*/24).bool(message.authorized);
            if (message.authExpireTime != null && Object.hasOwnProperty.call(message, "authExpireTime"))
                writer.uint32(/* id 4, wireType 0 =*/32).int64(message.authExpireTime);
            if (message.deviceDescription != null && Object.hasOwnProperty.call(message, "deviceDescription"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.deviceDescription);
            if (message.virusDbUpdatedAt != null && Object.hasOwnProperty.call(message, "virusDbUpdatedAt"))
                writer.uint32(/* id 6, wireType 0 =*/48).int64(message.virusDbUpdatedAt);
            if (message.authStatus != null && Object.hasOwnProperty.call(message, "authStatus"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.authStatus);
            return writer;
        };

        /**
         * Encodes the specified RspSystemInfo message, length delimited. Does not implicitly {@link usb_control.RspSystemInfo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspSystemInfo
         * @static
         * @param {usb_control.IRspSystemInfo} message RspSystemInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspSystemInfo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspSystemInfo message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspSystemInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspSystemInfo} RspSystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspSystemInfo.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspSystemInfo();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.systemVersion = reader.string();
                        break;
                    }
                case 2: {
                        message.virusDbVersion = reader.string();
                        break;
                    }
                case 3: {
                        message.authorized = reader.bool();
                        break;
                    }
                case 4: {
                        message.authExpireTime = reader.int64();
                        break;
                    }
                case 5: {
                        message.deviceDescription = reader.string();
                        break;
                    }
                case 6: {
                        message.virusDbUpdatedAt = reader.int64();
                        break;
                    }
                case 7: {
                        message.authStatus = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspSystemInfo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspSystemInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspSystemInfo} RspSystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspSystemInfo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspSystemInfo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspSystemInfo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspSystemInfo} RspSystemInfo
         */
        RspSystemInfo.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspSystemInfo)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspSystemInfo: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspSystemInfo();
            if (object.systemVersion != null)
                message.systemVersion = String(object.systemVersion);
            if (object.virusDbVersion != null)
                message.virusDbVersion = String(object.virusDbVersion);
            if (object.authorized != null)
                message.authorized = Boolean(object.authorized);
            if (object.authExpireTime != null)
                if ($util.Long)
                    message.authExpireTime = $util.Long.fromValue(object.authExpireTime, false);
                else if (typeof object.authExpireTime === "string")
                    message.authExpireTime = parseInt(object.authExpireTime, 10);
                else if (typeof object.authExpireTime === "number")
                    message.authExpireTime = object.authExpireTime;
                else if (typeof object.authExpireTime === "object")
                    message.authExpireTime = new $util.LongBits(object.authExpireTime.low >>> 0, object.authExpireTime.high >>> 0).toNumber();
            if (object.deviceDescription != null)
                message.deviceDescription = String(object.deviceDescription);
            if (object.virusDbUpdatedAt != null)
                if ($util.Long)
                    message.virusDbUpdatedAt = $util.Long.fromValue(object.virusDbUpdatedAt, false);
                else if (typeof object.virusDbUpdatedAt === "string")
                    message.virusDbUpdatedAt = parseInt(object.virusDbUpdatedAt, 10);
                else if (typeof object.virusDbUpdatedAt === "number")
                    message.virusDbUpdatedAt = object.virusDbUpdatedAt;
                else if (typeof object.virusDbUpdatedAt === "object")
                    message.virusDbUpdatedAt = new $util.LongBits(object.virusDbUpdatedAt.low >>> 0, object.virusDbUpdatedAt.high >>> 0).toNumber();
            if (object.authStatus != null)
                message.authStatus = String(object.authStatus);
            return message;
        };

        /**
         * Creates a plain object from a RspSystemInfo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspSystemInfo
         * @static
         * @param {usb_control.RspSystemInfo} message RspSystemInfo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspSystemInfo.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.systemVersion = "";
                object.virusDbVersion = "";
                object.authorized = false;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.authExpireTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.authExpireTime = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.deviceDescription = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.virusDbUpdatedAt = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.virusDbUpdatedAt = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                object.authStatus = "";
            }
            if (message.systemVersion != null && Object.hasOwnProperty.call(message, "systemVersion"))
                object.systemVersion = message.systemVersion;
            if (message.virusDbVersion != null && Object.hasOwnProperty.call(message, "virusDbVersion"))
                object.virusDbVersion = message.virusDbVersion;
            if (message.authorized != null && Object.hasOwnProperty.call(message, "authorized"))
                object.authorized = message.authorized;
            if (message.authExpireTime != null && Object.hasOwnProperty.call(message, "authExpireTime"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.authExpireTime = typeof message.authExpireTime === "number" ? BigInt(message.authExpireTime) : $util.Long.fromBits(message.authExpireTime.low >>> 0, message.authExpireTime.high >>> 0, false).toBigInt();
                else if (typeof message.authExpireTime === "number")
                    object.authExpireTime = options.longs === String ? String(message.authExpireTime) : message.authExpireTime;
                else
                    object.authExpireTime = options.longs === String ? $util.Long.prototype.toString.call(message.authExpireTime) : options.longs === Number ? new $util.LongBits(message.authExpireTime.low >>> 0, message.authExpireTime.high >>> 0).toNumber() : message.authExpireTime;
            if (message.deviceDescription != null && Object.hasOwnProperty.call(message, "deviceDescription"))
                object.deviceDescription = message.deviceDescription;
            if (message.virusDbUpdatedAt != null && Object.hasOwnProperty.call(message, "virusDbUpdatedAt"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.virusDbUpdatedAt = typeof message.virusDbUpdatedAt === "number" ? BigInt(message.virusDbUpdatedAt) : $util.Long.fromBits(message.virusDbUpdatedAt.low >>> 0, message.virusDbUpdatedAt.high >>> 0, false).toBigInt();
                else if (typeof message.virusDbUpdatedAt === "number")
                    object.virusDbUpdatedAt = options.longs === String ? String(message.virusDbUpdatedAt) : message.virusDbUpdatedAt;
                else
                    object.virusDbUpdatedAt = options.longs === String ? $util.Long.prototype.toString.call(message.virusDbUpdatedAt) : options.longs === Number ? new $util.LongBits(message.virusDbUpdatedAt.low >>> 0, message.virusDbUpdatedAt.high >>> 0).toNumber() : message.virusDbUpdatedAt;
            if (message.authStatus != null && Object.hasOwnProperty.call(message, "authStatus"))
                object.authStatus = message.authStatus;
            return object;
        };

        /**
         * Converts this RspSystemInfo to JSON.
         * @function toJSON
         * @memberof usb_control.RspSystemInfo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspSystemInfo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspSystemInfo
         * @function getTypeUrl
         * @memberof usb_control.RspSystemInfo
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspSystemInfo.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspSystemInfo";
        };

        return RspSystemInfo;
    })();

    usb_control.CmdUploadSystemUpgrade = (function() {

        /**
         * Properties of a CmdUploadSystemUpgrade.
         * @memberof usb_control
         * @interface ICmdUploadSystemUpgrade
         * @property {string|null} [sessionToken] CmdUploadSystemUpgrade sessionToken
         * @property {Uint8Array|null} [upgradeData] CmdUploadSystemUpgrade upgradeData
         * @property {string|null} [targetVersion] CmdUploadSystemUpgrade targetVersion
         * @property {string|null} [sha256Checksum] CmdUploadSystemUpgrade sha256Checksum
         */

        /**
         * Constructs a new CmdUploadSystemUpgrade.
         * @memberof usb_control
         * @classdesc Represents a CmdUploadSystemUpgrade.
         * @implements ICmdUploadSystemUpgrade
         * @constructor
         * @param {usb_control.ICmdUploadSystemUpgrade=} [properties] Properties to set
         */
        function CmdUploadSystemUpgrade(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdUploadSystemUpgrade sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @instance
         */
        CmdUploadSystemUpgrade.prototype.sessionToken = "";

        /**
         * CmdUploadSystemUpgrade upgradeData.
         * @member {Uint8Array} upgradeData
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @instance
         */
        CmdUploadSystemUpgrade.prototype.upgradeData = $util.newBuffer([]);

        /**
         * CmdUploadSystemUpgrade targetVersion.
         * @member {string} targetVersion
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @instance
         */
        CmdUploadSystemUpgrade.prototype.targetVersion = "";

        /**
         * CmdUploadSystemUpgrade sha256Checksum.
         * @member {string} sha256Checksum
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @instance
         */
        CmdUploadSystemUpgrade.prototype.sha256Checksum = "";

        /**
         * Encodes the specified CmdUploadSystemUpgrade message. Does not implicitly {@link usb_control.CmdUploadSystemUpgrade.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @static
         * @param {usb_control.ICmdUploadSystemUpgrade} message CmdUploadSystemUpgrade message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUploadSystemUpgrade.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.upgradeData != null && Object.hasOwnProperty.call(message, "upgradeData"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.upgradeData);
            if (message.targetVersion != null && Object.hasOwnProperty.call(message, "targetVersion"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.targetVersion);
            if (message.sha256Checksum != null && Object.hasOwnProperty.call(message, "sha256Checksum"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.sha256Checksum);
            return writer;
        };

        /**
         * Encodes the specified CmdUploadSystemUpgrade message, length delimited. Does not implicitly {@link usb_control.CmdUploadSystemUpgrade.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @static
         * @param {usb_control.ICmdUploadSystemUpgrade} message CmdUploadSystemUpgrade message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUploadSystemUpgrade.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdUploadSystemUpgrade message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdUploadSystemUpgrade} CmdUploadSystemUpgrade
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUploadSystemUpgrade.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdUploadSystemUpgrade();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.upgradeData = reader.bytes();
                        break;
                    }
                case 3: {
                        message.targetVersion = reader.string();
                        break;
                    }
                case 4: {
                        message.sha256Checksum = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdUploadSystemUpgrade message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdUploadSystemUpgrade} CmdUploadSystemUpgrade
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUploadSystemUpgrade.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdUploadSystemUpgrade message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdUploadSystemUpgrade} CmdUploadSystemUpgrade
         */
        CmdUploadSystemUpgrade.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdUploadSystemUpgrade)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdUploadSystemUpgrade: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdUploadSystemUpgrade();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.upgradeData != null)
                if (typeof object.upgradeData === "string")
                    $util.base64.decode(object.upgradeData, message.upgradeData = $util.newBuffer($util.base64.length(object.upgradeData)), 0);
                else if (object.upgradeData.length >= 0)
                    message.upgradeData = object.upgradeData;
            if (object.targetVersion != null)
                message.targetVersion = String(object.targetVersion);
            if (object.sha256Checksum != null)
                message.sha256Checksum = String(object.sha256Checksum);
            return message;
        };

        /**
         * Creates a plain object from a CmdUploadSystemUpgrade message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @static
         * @param {usb_control.CmdUploadSystemUpgrade} message CmdUploadSystemUpgrade
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdUploadSystemUpgrade.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                if (options.bytes === String)
                    object.upgradeData = "";
                else {
                    object.upgradeData = [];
                    if (options.bytes !== Array)
                        object.upgradeData = $util.newBuffer(object.upgradeData);
                }
                object.targetVersion = "";
                object.sha256Checksum = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.upgradeData != null && Object.hasOwnProperty.call(message, "upgradeData"))
                object.upgradeData = options.bytes === String ? $util.base64.encode(message.upgradeData, 0, message.upgradeData.length) : options.bytes === Array ? Array.prototype.slice.call(message.upgradeData) : message.upgradeData;
            if (message.targetVersion != null && Object.hasOwnProperty.call(message, "targetVersion"))
                object.targetVersion = message.targetVersion;
            if (message.sha256Checksum != null && Object.hasOwnProperty.call(message, "sha256Checksum"))
                object.sha256Checksum = message.sha256Checksum;
            return object;
        };

        /**
         * Converts this CmdUploadSystemUpgrade to JSON.
         * @function toJSON
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdUploadSystemUpgrade.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdUploadSystemUpgrade
         * @function getTypeUrl
         * @memberof usb_control.CmdUploadSystemUpgrade
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdUploadSystemUpgrade.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdUploadSystemUpgrade";
        };

        return CmdUploadSystemUpgrade;
    })();

    usb_control.CmdUploadVirusdbUpgrade = (function() {

        /**
         * Properties of a CmdUploadVirusdbUpgrade.
         * @memberof usb_control
         * @interface ICmdUploadVirusdbUpgrade
         * @property {string|null} [sessionToken] CmdUploadVirusdbUpgrade sessionToken
         * @property {Uint8Array|null} [upgradeData] CmdUploadVirusdbUpgrade upgradeData
         * @property {string|null} [targetVersion] CmdUploadVirusdbUpgrade targetVersion
         * @property {string|null} [sha256Checksum] CmdUploadVirusdbUpgrade sha256Checksum
         */

        /**
         * Constructs a new CmdUploadVirusdbUpgrade.
         * @memberof usb_control
         * @classdesc Represents a CmdUploadVirusdbUpgrade.
         * @implements ICmdUploadVirusdbUpgrade
         * @constructor
         * @param {usb_control.ICmdUploadVirusdbUpgrade=} [properties] Properties to set
         */
        function CmdUploadVirusdbUpgrade(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdUploadVirusdbUpgrade sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @instance
         */
        CmdUploadVirusdbUpgrade.prototype.sessionToken = "";

        /**
         * CmdUploadVirusdbUpgrade upgradeData.
         * @member {Uint8Array} upgradeData
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @instance
         */
        CmdUploadVirusdbUpgrade.prototype.upgradeData = $util.newBuffer([]);

        /**
         * CmdUploadVirusdbUpgrade targetVersion.
         * @member {string} targetVersion
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @instance
         */
        CmdUploadVirusdbUpgrade.prototype.targetVersion = "";

        /**
         * CmdUploadVirusdbUpgrade sha256Checksum.
         * @member {string} sha256Checksum
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @instance
         */
        CmdUploadVirusdbUpgrade.prototype.sha256Checksum = "";

        /**
         * Encodes the specified CmdUploadVirusdbUpgrade message. Does not implicitly {@link usb_control.CmdUploadVirusdbUpgrade.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @static
         * @param {usb_control.ICmdUploadVirusdbUpgrade} message CmdUploadVirusdbUpgrade message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUploadVirusdbUpgrade.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.upgradeData != null && Object.hasOwnProperty.call(message, "upgradeData"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.upgradeData);
            if (message.targetVersion != null && Object.hasOwnProperty.call(message, "targetVersion"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.targetVersion);
            if (message.sha256Checksum != null && Object.hasOwnProperty.call(message, "sha256Checksum"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.sha256Checksum);
            return writer;
        };

        /**
         * Encodes the specified CmdUploadVirusdbUpgrade message, length delimited. Does not implicitly {@link usb_control.CmdUploadVirusdbUpgrade.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @static
         * @param {usb_control.ICmdUploadVirusdbUpgrade} message CmdUploadVirusdbUpgrade message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUploadVirusdbUpgrade.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdUploadVirusdbUpgrade message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdUploadVirusdbUpgrade} CmdUploadVirusdbUpgrade
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUploadVirusdbUpgrade.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdUploadVirusdbUpgrade();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.upgradeData = reader.bytes();
                        break;
                    }
                case 3: {
                        message.targetVersion = reader.string();
                        break;
                    }
                case 4: {
                        message.sha256Checksum = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdUploadVirusdbUpgrade message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdUploadVirusdbUpgrade} CmdUploadVirusdbUpgrade
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUploadVirusdbUpgrade.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdUploadVirusdbUpgrade message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdUploadVirusdbUpgrade} CmdUploadVirusdbUpgrade
         */
        CmdUploadVirusdbUpgrade.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdUploadVirusdbUpgrade)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdUploadVirusdbUpgrade: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdUploadVirusdbUpgrade();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.upgradeData != null)
                if (typeof object.upgradeData === "string")
                    $util.base64.decode(object.upgradeData, message.upgradeData = $util.newBuffer($util.base64.length(object.upgradeData)), 0);
                else if (object.upgradeData.length >= 0)
                    message.upgradeData = object.upgradeData;
            if (object.targetVersion != null)
                message.targetVersion = String(object.targetVersion);
            if (object.sha256Checksum != null)
                message.sha256Checksum = String(object.sha256Checksum);
            return message;
        };

        /**
         * Creates a plain object from a CmdUploadVirusdbUpgrade message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @static
         * @param {usb_control.CmdUploadVirusdbUpgrade} message CmdUploadVirusdbUpgrade
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdUploadVirusdbUpgrade.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                if (options.bytes === String)
                    object.upgradeData = "";
                else {
                    object.upgradeData = [];
                    if (options.bytes !== Array)
                        object.upgradeData = $util.newBuffer(object.upgradeData);
                }
                object.targetVersion = "";
                object.sha256Checksum = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.upgradeData != null && Object.hasOwnProperty.call(message, "upgradeData"))
                object.upgradeData = options.bytes === String ? $util.base64.encode(message.upgradeData, 0, message.upgradeData.length) : options.bytes === Array ? Array.prototype.slice.call(message.upgradeData) : message.upgradeData;
            if (message.targetVersion != null && Object.hasOwnProperty.call(message, "targetVersion"))
                object.targetVersion = message.targetVersion;
            if (message.sha256Checksum != null && Object.hasOwnProperty.call(message, "sha256Checksum"))
                object.sha256Checksum = message.sha256Checksum;
            return object;
        };

        /**
         * Converts this CmdUploadVirusdbUpgrade to JSON.
         * @function toJSON
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdUploadVirusdbUpgrade.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdUploadVirusdbUpgrade
         * @function getTypeUrl
         * @memberof usb_control.CmdUploadVirusdbUpgrade
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdUploadVirusdbUpgrade.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdUploadVirusdbUpgrade";
        };

        return CmdUploadVirusdbUpgrade;
    })();

    usb_control.CmdUpdateDeviceDesc = (function() {

        /**
         * Properties of a CmdUpdateDeviceDesc.
         * @memberof usb_control
         * @interface ICmdUpdateDeviceDesc
         * @property {string|null} [sessionToken] CmdUpdateDeviceDesc sessionToken
         * @property {string|null} [description] CmdUpdateDeviceDesc description
         */

        /**
         * Constructs a new CmdUpdateDeviceDesc.
         * @memberof usb_control
         * @classdesc Represents a CmdUpdateDeviceDesc.
         * @implements ICmdUpdateDeviceDesc
         * @constructor
         * @param {usb_control.ICmdUpdateDeviceDesc=} [properties] Properties to set
         */
        function CmdUpdateDeviceDesc(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdUpdateDeviceDesc sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @instance
         */
        CmdUpdateDeviceDesc.prototype.sessionToken = "";

        /**
         * CmdUpdateDeviceDesc description.
         * @member {string} description
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @instance
         */
        CmdUpdateDeviceDesc.prototype.description = "";

        /**
         * Encodes the specified CmdUpdateDeviceDesc message. Does not implicitly {@link usb_control.CmdUpdateDeviceDesc.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @static
         * @param {usb_control.ICmdUpdateDeviceDesc} message CmdUpdateDeviceDesc message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUpdateDeviceDesc.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.description);
            return writer;
        };

        /**
         * Encodes the specified CmdUpdateDeviceDesc message, length delimited. Does not implicitly {@link usb_control.CmdUpdateDeviceDesc.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @static
         * @param {usb_control.ICmdUpdateDeviceDesc} message CmdUpdateDeviceDesc message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdUpdateDeviceDesc.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdUpdateDeviceDesc message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdUpdateDeviceDesc} CmdUpdateDeviceDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUpdateDeviceDesc.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdUpdateDeviceDesc();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.description = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdUpdateDeviceDesc message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdUpdateDeviceDesc} CmdUpdateDeviceDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdUpdateDeviceDesc.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdUpdateDeviceDesc message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdUpdateDeviceDesc} CmdUpdateDeviceDesc
         */
        CmdUpdateDeviceDesc.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdUpdateDeviceDesc)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdUpdateDeviceDesc: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdUpdateDeviceDesc();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.description != null)
                message.description = String(object.description);
            return message;
        };

        /**
         * Creates a plain object from a CmdUpdateDeviceDesc message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @static
         * @param {usb_control.CmdUpdateDeviceDesc} message CmdUpdateDeviceDesc
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdUpdateDeviceDesc.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.description = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                object.description = message.description;
            return object;
        };

        /**
         * Converts this CmdUpdateDeviceDesc to JSON.
         * @function toJSON
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdUpdateDeviceDesc.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdUpdateDeviceDesc
         * @function getTypeUrl
         * @memberof usb_control.CmdUpdateDeviceDesc
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdUpdateDeviceDesc.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdUpdateDeviceDesc";
        };

        return CmdUpdateDeviceDesc;
    })();

    usb_control.UserItem = (function() {

        /**
         * Properties of a UserItem.
         * @memberof usb_control
         * @interface IUserItem
         * @property {string|null} [username] UserItem username
         * @property {string|null} [role] UserItem role
         * @property {string|null} [status] UserItem status
         * @property {boolean|null} [isBuiltin] UserItem isBuiltin
         * @property {number|Long|null} [createdAt] UserItem createdAt
         */

        /**
         * Constructs a new UserItem.
         * @memberof usb_control
         * @classdesc Represents a UserItem.
         * @implements IUserItem
         * @constructor
         * @param {usb_control.IUserItem=} [properties] Properties to set
         */
        function UserItem(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * UserItem username.
         * @member {string} username
         * @memberof usb_control.UserItem
         * @instance
         */
        UserItem.prototype.username = "";

        /**
         * UserItem role.
         * @member {string} role
         * @memberof usb_control.UserItem
         * @instance
         */
        UserItem.prototype.role = "";

        /**
         * UserItem status.
         * @member {string} status
         * @memberof usb_control.UserItem
         * @instance
         */
        UserItem.prototype.status = "";

        /**
         * UserItem isBuiltin.
         * @member {boolean} isBuiltin
         * @memberof usb_control.UserItem
         * @instance
         */
        UserItem.prototype.isBuiltin = false;

        /**
         * UserItem createdAt.
         * @member {number|Long} createdAt
         * @memberof usb_control.UserItem
         * @instance
         */
        UserItem.prototype.createdAt = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * Encodes the specified UserItem message. Does not implicitly {@link usb_control.UserItem.verify|verify} messages.
         * @function encode
         * @memberof usb_control.UserItem
         * @static
         * @param {usb_control.IUserItem} message UserItem message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        UserItem.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.username);
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.role);
            if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.status);
            if (message.isBuiltin != null && Object.hasOwnProperty.call(message, "isBuiltin"))
                writer.uint32(/* id 4, wireType 0 =*/32).bool(message.isBuiltin);
            if (message.createdAt != null && Object.hasOwnProperty.call(message, "createdAt"))
                writer.uint32(/* id 5, wireType 0 =*/40).int64(message.createdAt);
            return writer;
        };

        /**
         * Encodes the specified UserItem message, length delimited. Does not implicitly {@link usb_control.UserItem.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.UserItem
         * @static
         * @param {usb_control.IUserItem} message UserItem message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        UserItem.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a UserItem message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.UserItem
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.UserItem} UserItem
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        UserItem.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.UserItem();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.username = reader.string();
                        break;
                    }
                case 2: {
                        message.role = reader.string();
                        break;
                    }
                case 3: {
                        message.status = reader.string();
                        break;
                    }
                case 4: {
                        message.isBuiltin = reader.bool();
                        break;
                    }
                case 5: {
                        message.createdAt = reader.int64();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a UserItem message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.UserItem
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.UserItem} UserItem
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        UserItem.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a UserItem message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.UserItem
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.UserItem} UserItem
         */
        UserItem.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.UserItem)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.UserItem: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.UserItem();
            if (object.username != null)
                message.username = String(object.username);
            if (object.role != null)
                message.role = String(object.role);
            if (object.status != null)
                message.status = String(object.status);
            if (object.isBuiltin != null)
                message.isBuiltin = Boolean(object.isBuiltin);
            if (object.createdAt != null)
                if ($util.Long)
                    message.createdAt = $util.Long.fromValue(object.createdAt, false);
                else if (typeof object.createdAt === "string")
                    message.createdAt = parseInt(object.createdAt, 10);
                else if (typeof object.createdAt === "number")
                    message.createdAt = object.createdAt;
                else if (typeof object.createdAt === "object")
                    message.createdAt = new $util.LongBits(object.createdAt.low >>> 0, object.createdAt.high >>> 0).toNumber();
            return message;
        };

        /**
         * Creates a plain object from a UserItem message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.UserItem
         * @static
         * @param {usb_control.UserItem} message UserItem
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        UserItem.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.username = "";
                object.role = "";
                object.status = "";
                object.isBuiltin = false;
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.createdAt = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                } else
                    object.createdAt = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
            }
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                object.username = message.username;
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                object.role = message.role;
            if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                object.status = message.status;
            if (message.isBuiltin != null && Object.hasOwnProperty.call(message, "isBuiltin"))
                object.isBuiltin = message.isBuiltin;
            if (message.createdAt != null && Object.hasOwnProperty.call(message, "createdAt"))
                if (typeof BigInt !== "undefined" && options.longs === BigInt)
                    object.createdAt = typeof message.createdAt === "number" ? BigInt(message.createdAt) : $util.Long.fromBits(message.createdAt.low >>> 0, message.createdAt.high >>> 0, false).toBigInt();
                else if (typeof message.createdAt === "number")
                    object.createdAt = options.longs === String ? String(message.createdAt) : message.createdAt;
                else
                    object.createdAt = options.longs === String ? $util.Long.prototype.toString.call(message.createdAt) : options.longs === Number ? new $util.LongBits(message.createdAt.low >>> 0, message.createdAt.high >>> 0).toNumber() : message.createdAt;
            return object;
        };

        /**
         * Converts this UserItem to JSON.
         * @function toJSON
         * @memberof usb_control.UserItem
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        UserItem.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for UserItem
         * @function getTypeUrl
         * @memberof usb_control.UserItem
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        UserItem.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.UserItem";
        };

        return UserItem;
    })();

    usb_control.CmdListUsers = (function() {

        /**
         * Properties of a CmdListUsers.
         * @memberof usb_control
         * @interface ICmdListUsers
         * @property {string|null} [sessionToken] CmdListUsers sessionToken
         */

        /**
         * Constructs a new CmdListUsers.
         * @memberof usb_control
         * @classdesc Represents a CmdListUsers.
         * @implements ICmdListUsers
         * @constructor
         * @param {usb_control.ICmdListUsers=} [properties] Properties to set
         */
        function CmdListUsers(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdListUsers sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdListUsers
         * @instance
         */
        CmdListUsers.prototype.sessionToken = "";

        /**
         * Encodes the specified CmdListUsers message. Does not implicitly {@link usb_control.CmdListUsers.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdListUsers
         * @static
         * @param {usb_control.ICmdListUsers} message CmdListUsers message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdListUsers.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            return writer;
        };

        /**
         * Encodes the specified CmdListUsers message, length delimited. Does not implicitly {@link usb_control.CmdListUsers.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdListUsers
         * @static
         * @param {usb_control.ICmdListUsers} message CmdListUsers message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdListUsers.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdListUsers message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdListUsers
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdListUsers} CmdListUsers
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdListUsers.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdListUsers();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdListUsers message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdListUsers
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdListUsers} CmdListUsers
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdListUsers.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdListUsers message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdListUsers
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdListUsers} CmdListUsers
         */
        CmdListUsers.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdListUsers)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdListUsers: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdListUsers();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            return message;
        };

        /**
         * Creates a plain object from a CmdListUsers message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdListUsers
         * @static
         * @param {usb_control.CmdListUsers} message CmdListUsers
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdListUsers.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults)
                object.sessionToken = "";
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            return object;
        };

        /**
         * Converts this CmdListUsers to JSON.
         * @function toJSON
         * @memberof usb_control.CmdListUsers
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdListUsers.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdListUsers
         * @function getTypeUrl
         * @memberof usb_control.CmdListUsers
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdListUsers.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdListUsers";
        };

        return CmdListUsers;
    })();

    usb_control.RspListUsers = (function() {

        /**
         * Properties of a RspListUsers.
         * @memberof usb_control
         * @interface IRspListUsers
         * @property {Array.<usb_control.IUserItem>|null} [users] RspListUsers users
         */

        /**
         * Constructs a new RspListUsers.
         * @memberof usb_control
         * @classdesc Represents a RspListUsers.
         * @implements IRspListUsers
         * @constructor
         * @param {usb_control.IRspListUsers=} [properties] Properties to set
         */
        function RspListUsers(properties) {
            this.users = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspListUsers users.
         * @member {Array.<usb_control.IUserItem>} users
         * @memberof usb_control.RspListUsers
         * @instance
         */
        RspListUsers.prototype.users = $util.emptyArray;

        /**
         * Encodes the specified RspListUsers message. Does not implicitly {@link usb_control.RspListUsers.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspListUsers
         * @static
         * @param {usb_control.IRspListUsers} message RspListUsers message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspListUsers.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.users != null && message.users.length)
                for (let i = 0; i < message.users.length; ++i)
                    $root.usb_control.UserItem.encode(message.users[i], writer.uint32(/* id 1, wireType 2 =*/10).fork(), q + 1).ldelim();
            return writer;
        };

        /**
         * Encodes the specified RspListUsers message, length delimited. Does not implicitly {@link usb_control.RspListUsers.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspListUsers
         * @static
         * @param {usb_control.IRspListUsers} message RspListUsers message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspListUsers.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspListUsers message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspListUsers
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspListUsers} RspListUsers
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspListUsers.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspListUsers();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        if (!(message.users && message.users.length))
                            message.users = [];
                        message.users.push($root.usb_control.UserItem.decode(reader, reader.uint32(), undefined, long + 1));
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspListUsers message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspListUsers
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspListUsers} RspListUsers
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspListUsers.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspListUsers message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspListUsers
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspListUsers} RspListUsers
         */
        RspListUsers.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspListUsers)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspListUsers: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspListUsers();
            if (object.users) {
                if (!Array.isArray(object.users))
                    throw TypeError(".usb_control.RspListUsers.users: array expected");
                message.users = [];
                for (let i = 0; i < object.users.length; ++i) {
                    if (!$util.isObject(object.users[i]))
                        throw TypeError(".usb_control.RspListUsers.users: object expected");
                    message.users[i] = $root.usb_control.UserItem.fromObject(object.users[i], long + 1);
                }
            }
            return message;
        };

        /**
         * Creates a plain object from a RspListUsers message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspListUsers
         * @static
         * @param {usb_control.RspListUsers} message RspListUsers
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspListUsers.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.arrays || options.defaults)
                object.users = [];
            if (message.users && message.users.length) {
                object.users = [];
                for (let j = 0; j < message.users.length; ++j)
                    object.users[j] = $root.usb_control.UserItem.toObject(message.users[j], options, q + 1);
            }
            return object;
        };

        /**
         * Converts this RspListUsers to JSON.
         * @function toJSON
         * @memberof usb_control.RspListUsers
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspListUsers.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspListUsers
         * @function getTypeUrl
         * @memberof usb_control.RspListUsers
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspListUsers.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspListUsers";
        };

        return RspListUsers;
    })();

    usb_control.CmdCreateUser = (function() {

        /**
         * Properties of a CmdCreateUser.
         * @memberof usb_control
         * @interface ICmdCreateUser
         * @property {string|null} [sessionToken] CmdCreateUser sessionToken
         * @property {string|null} [username] CmdCreateUser username
         * @property {string|null} [role] CmdCreateUser role
         * @property {string|null} [password] CmdCreateUser password
         * @property {string|null} [confirmPassword] CmdCreateUser confirmPassword
         */

        /**
         * Constructs a new CmdCreateUser.
         * @memberof usb_control
         * @classdesc Represents a CmdCreateUser.
         * @implements ICmdCreateUser
         * @constructor
         * @param {usb_control.ICmdCreateUser=} [properties] Properties to set
         */
        function CmdCreateUser(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdCreateUser sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdCreateUser
         * @instance
         */
        CmdCreateUser.prototype.sessionToken = "";

        /**
         * CmdCreateUser username.
         * @member {string} username
         * @memberof usb_control.CmdCreateUser
         * @instance
         */
        CmdCreateUser.prototype.username = "";

        /**
         * CmdCreateUser role.
         * @member {string} role
         * @memberof usb_control.CmdCreateUser
         * @instance
         */
        CmdCreateUser.prototype.role = "";

        /**
         * CmdCreateUser password.
         * @member {string} password
         * @memberof usb_control.CmdCreateUser
         * @instance
         */
        CmdCreateUser.prototype.password = "";

        /**
         * CmdCreateUser confirmPassword.
         * @member {string} confirmPassword
         * @memberof usb_control.CmdCreateUser
         * @instance
         */
        CmdCreateUser.prototype.confirmPassword = "";

        /**
         * Encodes the specified CmdCreateUser message. Does not implicitly {@link usb_control.CmdCreateUser.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdCreateUser
         * @static
         * @param {usb_control.ICmdCreateUser} message CmdCreateUser message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdCreateUser.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.username);
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.role);
            if (message.password != null && Object.hasOwnProperty.call(message, "password"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.password);
            if (message.confirmPassword != null && Object.hasOwnProperty.call(message, "confirmPassword"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.confirmPassword);
            return writer;
        };

        /**
         * Encodes the specified CmdCreateUser message, length delimited. Does not implicitly {@link usb_control.CmdCreateUser.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdCreateUser
         * @static
         * @param {usb_control.ICmdCreateUser} message CmdCreateUser message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdCreateUser.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdCreateUser message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdCreateUser
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdCreateUser} CmdCreateUser
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdCreateUser.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdCreateUser();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.username = reader.string();
                        break;
                    }
                case 3: {
                        message.role = reader.string();
                        break;
                    }
                case 4: {
                        message.password = reader.string();
                        break;
                    }
                case 5: {
                        message.confirmPassword = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdCreateUser message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdCreateUser
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdCreateUser} CmdCreateUser
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdCreateUser.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdCreateUser message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdCreateUser
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdCreateUser} CmdCreateUser
         */
        CmdCreateUser.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdCreateUser)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdCreateUser: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdCreateUser();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.username != null)
                message.username = String(object.username);
            if (object.role != null)
                message.role = String(object.role);
            if (object.password != null)
                message.password = String(object.password);
            if (object.confirmPassword != null)
                message.confirmPassword = String(object.confirmPassword);
            return message;
        };

        /**
         * Creates a plain object from a CmdCreateUser message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdCreateUser
         * @static
         * @param {usb_control.CmdCreateUser} message CmdCreateUser
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdCreateUser.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.username = "";
                object.role = "";
                object.password = "";
                object.confirmPassword = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                object.username = message.username;
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                object.role = message.role;
            if (message.password != null && Object.hasOwnProperty.call(message, "password"))
                object.password = message.password;
            if (message.confirmPassword != null && Object.hasOwnProperty.call(message, "confirmPassword"))
                object.confirmPassword = message.confirmPassword;
            return object;
        };

        /**
         * Converts this CmdCreateUser to JSON.
         * @function toJSON
         * @memberof usb_control.CmdCreateUser
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdCreateUser.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdCreateUser
         * @function getTypeUrl
         * @memberof usb_control.CmdCreateUser
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdCreateUser.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdCreateUser";
        };

        return CmdCreateUser;
    })();

    usb_control.CmdDeleteUser = (function() {

        /**
         * Properties of a CmdDeleteUser.
         * @memberof usb_control
         * @interface ICmdDeleteUser
         * @property {string|null} [sessionToken] CmdDeleteUser sessionToken
         * @property {string|null} [username] CmdDeleteUser username
         */

        /**
         * Constructs a new CmdDeleteUser.
         * @memberof usb_control
         * @classdesc Represents a CmdDeleteUser.
         * @implements ICmdDeleteUser
         * @constructor
         * @param {usb_control.ICmdDeleteUser=} [properties] Properties to set
         */
        function CmdDeleteUser(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdDeleteUser sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdDeleteUser
         * @instance
         */
        CmdDeleteUser.prototype.sessionToken = "";

        /**
         * CmdDeleteUser username.
         * @member {string} username
         * @memberof usb_control.CmdDeleteUser
         * @instance
         */
        CmdDeleteUser.prototype.username = "";

        /**
         * Encodes the specified CmdDeleteUser message. Does not implicitly {@link usb_control.CmdDeleteUser.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdDeleteUser
         * @static
         * @param {usb_control.ICmdDeleteUser} message CmdDeleteUser message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdDeleteUser.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.username);
            return writer;
        };

        /**
         * Encodes the specified CmdDeleteUser message, length delimited. Does not implicitly {@link usb_control.CmdDeleteUser.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdDeleteUser
         * @static
         * @param {usb_control.ICmdDeleteUser} message CmdDeleteUser message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdDeleteUser.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdDeleteUser message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdDeleteUser
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdDeleteUser} CmdDeleteUser
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdDeleteUser.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdDeleteUser();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.username = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdDeleteUser message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdDeleteUser
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdDeleteUser} CmdDeleteUser
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdDeleteUser.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdDeleteUser message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdDeleteUser
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdDeleteUser} CmdDeleteUser
         */
        CmdDeleteUser.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdDeleteUser)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdDeleteUser: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdDeleteUser();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.username != null)
                message.username = String(object.username);
            return message;
        };

        /**
         * Creates a plain object from a CmdDeleteUser message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdDeleteUser
         * @static
         * @param {usb_control.CmdDeleteUser} message CmdDeleteUser
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdDeleteUser.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.username = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                object.username = message.username;
            return object;
        };

        /**
         * Converts this CmdDeleteUser to JSON.
         * @function toJSON
         * @memberof usb_control.CmdDeleteUser
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdDeleteUser.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdDeleteUser
         * @function getTypeUrl
         * @memberof usb_control.CmdDeleteUser
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdDeleteUser.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdDeleteUser";
        };

        return CmdDeleteUser;
    })();

    usb_control.CmdResetPassword = (function() {

        /**
         * Properties of a CmdResetPassword.
         * @memberof usb_control
         * @interface ICmdResetPassword
         * @property {string|null} [sessionToken] CmdResetPassword sessionToken
         * @property {string|null} [username] CmdResetPassword username
         * @property {string|null} [newPassword] CmdResetPassword newPassword
         * @property {string|null} [confirmPassword] CmdResetPassword confirmPassword
         */

        /**
         * Constructs a new CmdResetPassword.
         * @memberof usb_control
         * @classdesc Represents a CmdResetPassword.
         * @implements ICmdResetPassword
         * @constructor
         * @param {usb_control.ICmdResetPassword=} [properties] Properties to set
         */
        function CmdResetPassword(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdResetPassword sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdResetPassword
         * @instance
         */
        CmdResetPassword.prototype.sessionToken = "";

        /**
         * CmdResetPassword username.
         * @member {string} username
         * @memberof usb_control.CmdResetPassword
         * @instance
         */
        CmdResetPassword.prototype.username = "";

        /**
         * CmdResetPassword newPassword.
         * @member {string} newPassword
         * @memberof usb_control.CmdResetPassword
         * @instance
         */
        CmdResetPassword.prototype.newPassword = "";

        /**
         * CmdResetPassword confirmPassword.
         * @member {string} confirmPassword
         * @memberof usb_control.CmdResetPassword
         * @instance
         */
        CmdResetPassword.prototype.confirmPassword = "";

        /**
         * Encodes the specified CmdResetPassword message. Does not implicitly {@link usb_control.CmdResetPassword.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdResetPassword
         * @static
         * @param {usb_control.ICmdResetPassword} message CmdResetPassword message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdResetPassword.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.username);
            if (message.newPassword != null && Object.hasOwnProperty.call(message, "newPassword"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.newPassword);
            if (message.confirmPassword != null && Object.hasOwnProperty.call(message, "confirmPassword"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.confirmPassword);
            return writer;
        };

        /**
         * Encodes the specified CmdResetPassword message, length delimited. Does not implicitly {@link usb_control.CmdResetPassword.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdResetPassword
         * @static
         * @param {usb_control.ICmdResetPassword} message CmdResetPassword message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdResetPassword.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdResetPassword message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdResetPassword
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdResetPassword} CmdResetPassword
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdResetPassword.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdResetPassword();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.username = reader.string();
                        break;
                    }
                case 3: {
                        message.newPassword = reader.string();
                        break;
                    }
                case 4: {
                        message.confirmPassword = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdResetPassword message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdResetPassword
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdResetPassword} CmdResetPassword
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdResetPassword.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdResetPassword message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdResetPassword
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdResetPassword} CmdResetPassword
         */
        CmdResetPassword.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdResetPassword)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdResetPassword: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdResetPassword();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.username != null)
                message.username = String(object.username);
            if (object.newPassword != null)
                message.newPassword = String(object.newPassword);
            if (object.confirmPassword != null)
                message.confirmPassword = String(object.confirmPassword);
            return message;
        };

        /**
         * Creates a plain object from a CmdResetPassword message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdResetPassword
         * @static
         * @param {usb_control.CmdResetPassword} message CmdResetPassword
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdResetPassword.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.username = "";
                object.newPassword = "";
                object.confirmPassword = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                object.username = message.username;
            if (message.newPassword != null && Object.hasOwnProperty.call(message, "newPassword"))
                object.newPassword = message.newPassword;
            if (message.confirmPassword != null && Object.hasOwnProperty.call(message, "confirmPassword"))
                object.confirmPassword = message.confirmPassword;
            return object;
        };

        /**
         * Converts this CmdResetPassword to JSON.
         * @function toJSON
         * @memberof usb_control.CmdResetPassword
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdResetPassword.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdResetPassword
         * @function getTypeUrl
         * @memberof usb_control.CmdResetPassword
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdResetPassword.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdResetPassword";
        };

        return CmdResetPassword;
    })();

    usb_control.CmdChangePassword = (function() {

        /**
         * Properties of a CmdChangePassword.
         * @memberof usb_control
         * @interface ICmdChangePassword
         * @property {string|null} [sessionToken] CmdChangePassword sessionToken
         * @property {string|null} [oldPassword] CmdChangePassword oldPassword
         * @property {string|null} [newPassword] CmdChangePassword newPassword
         * @property {string|null} [confirmPassword] CmdChangePassword confirmPassword
         */

        /**
         * Constructs a new CmdChangePassword.
         * @memberof usb_control
         * @classdesc Represents a CmdChangePassword.
         * @implements ICmdChangePassword
         * @constructor
         * @param {usb_control.ICmdChangePassword=} [properties] Properties to set
         */
        function CmdChangePassword(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CmdChangePassword sessionToken.
         * @member {string} sessionToken
         * @memberof usb_control.CmdChangePassword
         * @instance
         */
        CmdChangePassword.prototype.sessionToken = "";

        /**
         * CmdChangePassword oldPassword.
         * @member {string} oldPassword
         * @memberof usb_control.CmdChangePassword
         * @instance
         */
        CmdChangePassword.prototype.oldPassword = "";

        /**
         * CmdChangePassword newPassword.
         * @member {string} newPassword
         * @memberof usb_control.CmdChangePassword
         * @instance
         */
        CmdChangePassword.prototype.newPassword = "";

        /**
         * CmdChangePassword confirmPassword.
         * @member {string} confirmPassword
         * @memberof usb_control.CmdChangePassword
         * @instance
         */
        CmdChangePassword.prototype.confirmPassword = "";

        /**
         * Encodes the specified CmdChangePassword message. Does not implicitly {@link usb_control.CmdChangePassword.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdChangePassword
         * @static
         * @param {usb_control.ICmdChangePassword} message CmdChangePassword message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdChangePassword.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.sessionToken);
            if (message.oldPassword != null && Object.hasOwnProperty.call(message, "oldPassword"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.oldPassword);
            if (message.newPassword != null && Object.hasOwnProperty.call(message, "newPassword"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.newPassword);
            if (message.confirmPassword != null && Object.hasOwnProperty.call(message, "confirmPassword"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.confirmPassword);
            return writer;
        };

        /**
         * Encodes the specified CmdChangePassword message, length delimited. Does not implicitly {@link usb_control.CmdChangePassword.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdChangePassword
         * @static
         * @param {usb_control.ICmdChangePassword} message CmdChangePassword message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdChangePassword.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdChangePassword message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdChangePassword
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdChangePassword} CmdChangePassword
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdChangePassword.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdChangePassword();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.sessionToken = reader.string();
                        break;
                    }
                case 2: {
                        message.oldPassword = reader.string();
                        break;
                    }
                case 3: {
                        message.newPassword = reader.string();
                        break;
                    }
                case 4: {
                        message.confirmPassword = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdChangePassword message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdChangePassword
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdChangePassword} CmdChangePassword
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdChangePassword.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdChangePassword message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdChangePassword
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdChangePassword} CmdChangePassword
         */
        CmdChangePassword.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdChangePassword)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.CmdChangePassword: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.CmdChangePassword();
            if (object.sessionToken != null)
                message.sessionToken = String(object.sessionToken);
            if (object.oldPassword != null)
                message.oldPassword = String(object.oldPassword);
            if (object.newPassword != null)
                message.newPassword = String(object.newPassword);
            if (object.confirmPassword != null)
                message.confirmPassword = String(object.confirmPassword);
            return message;
        };

        /**
         * Creates a plain object from a CmdChangePassword message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdChangePassword
         * @static
         * @param {usb_control.CmdChangePassword} message CmdChangePassword
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdChangePassword.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.sessionToken = "";
                object.oldPassword = "";
                object.newPassword = "";
                object.confirmPassword = "";
            }
            if (message.sessionToken != null && Object.hasOwnProperty.call(message, "sessionToken"))
                object.sessionToken = message.sessionToken;
            if (message.oldPassword != null && Object.hasOwnProperty.call(message, "oldPassword"))
                object.oldPassword = message.oldPassword;
            if (message.newPassword != null && Object.hasOwnProperty.call(message, "newPassword"))
                object.newPassword = message.newPassword;
            if (message.confirmPassword != null && Object.hasOwnProperty.call(message, "confirmPassword"))
                object.confirmPassword = message.confirmPassword;
            return object;
        };

        /**
         * Converts this CmdChangePassword to JSON.
         * @function toJSON
         * @memberof usb_control.CmdChangePassword
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdChangePassword.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdChangePassword
         * @function getTypeUrl
         * @memberof usb_control.CmdChangePassword
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdChangePassword.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdChangePassword";
        };

        return CmdChangePassword;
    })();

    usb_control.RspCommon = (function() {

        /**
         * Properties of a RspCommon.
         * @memberof usb_control
         * @interface IRspCommon
         * @property {boolean|null} [success] RspCommon success
         * @property {number|null} [resultCode] RspCommon resultCode
         * @property {string|null} [errorMessage] RspCommon errorMessage
         */

        /**
         * Constructs a new RspCommon.
         * @memberof usb_control
         * @classdesc Represents a RspCommon.
         * @implements IRspCommon
         * @constructor
         * @param {usb_control.IRspCommon=} [properties] Properties to set
         */
        function RspCommon(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RspCommon success.
         * @member {boolean} success
         * @memberof usb_control.RspCommon
         * @instance
         */
        RspCommon.prototype.success = false;

        /**
         * RspCommon resultCode.
         * @member {number} resultCode
         * @memberof usb_control.RspCommon
         * @instance
         */
        RspCommon.prototype.resultCode = 0;

        /**
         * RspCommon errorMessage.
         * @member {string} errorMessage
         * @memberof usb_control.RspCommon
         * @instance
         */
        RspCommon.prototype.errorMessage = "";

        /**
         * Encodes the specified RspCommon message. Does not implicitly {@link usb_control.RspCommon.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspCommon
         * @static
         * @param {usb_control.IRspCommon} message RspCommon message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspCommon.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                writer.uint32(/* id 2, wireType 0 =*/16).int32(message.resultCode);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.errorMessage);
            return writer;
        };

        /**
         * Encodes the specified RspCommon message, length delimited. Does not implicitly {@link usb_control.RspCommon.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspCommon
         * @static
         * @param {usb_control.IRspCommon} message RspCommon message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspCommon.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspCommon message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspCommon
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspCommon} RspCommon
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspCommon.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspCommon();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.success = reader.bool();
                        break;
                    }
                case 2: {
                        message.resultCode = reader.int32();
                        break;
                    }
                case 3: {
                        message.errorMessage = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspCommon message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspCommon
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspCommon} RspCommon
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspCommon.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspCommon message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspCommon
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspCommon} RspCommon
         */
        RspCommon.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspCommon)
                return object;
            if (!$util.isObject(object))
                throw TypeError(".usb_control.RspCommon: object expected");
            if (long === undefined)
                long = 0;
            if (long > $util.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let message = new $root.usb_control.RspCommon();
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.resultCode != null)
                message.resultCode = object.resultCode | 0;
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            return message;
        };

        /**
         * Creates a plain object from a RspCommon message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspCommon
         * @static
         * @param {usb_control.RspCommon} message RspCommon
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspCommon.toObject = function toObject(message, options, q) {
            if (!options)
                options = {};
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            let object = {};
            if (options.defaults) {
                object.success = false;
                object.resultCode = 0;
                object.errorMessage = "";
            }
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                object.success = message.success;
            if (message.resultCode != null && Object.hasOwnProperty.call(message, "resultCode"))
                object.resultCode = message.resultCode;
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                object.errorMessage = message.errorMessage;
            return object;
        };

        /**
         * Converts this RspCommon to JSON.
         * @function toJSON
         * @memberof usb_control.RspCommon
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspCommon.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspCommon
         * @function getTypeUrl
         * @memberof usb_control.RspCommon
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspCommon.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspCommon";
        };

        return RspCommon;
    })();

    usb_control.CmdHeartbeat = (function() {

        /**
         * Properties of a CmdHeartbeat.
         * @memberof usb_control
         * @interface ICmdHeartbeat
         */

        /**
         * Constructs a new CmdHeartbeat.
         * @memberof usb_control
         * @classdesc Represents a CmdHeartbeat.
         * @implements ICmdHeartbeat
         * @constructor
         * @param {usb_control.ICmdHeartbeat=} [properties] Properties to set
         */
        function CmdHeartbeat(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * Encodes the specified CmdHeartbeat message. Does not implicitly {@link usb_control.CmdHeartbeat.verify|verify} messages.
         * @function encode
         * @memberof usb_control.CmdHeartbeat
         * @static
         * @param {usb_control.ICmdHeartbeat} message CmdHeartbeat message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdHeartbeat.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            return writer;
        };

        /**
         * Encodes the specified CmdHeartbeat message, length delimited. Does not implicitly {@link usb_control.CmdHeartbeat.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.CmdHeartbeat
         * @static
         * @param {usb_control.ICmdHeartbeat} message CmdHeartbeat message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CmdHeartbeat.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a CmdHeartbeat message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.CmdHeartbeat
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.CmdHeartbeat} CmdHeartbeat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdHeartbeat.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.CmdHeartbeat();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CmdHeartbeat message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.CmdHeartbeat
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.CmdHeartbeat} CmdHeartbeat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CmdHeartbeat.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a CmdHeartbeat message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.CmdHeartbeat
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.CmdHeartbeat} CmdHeartbeat
         */
        CmdHeartbeat.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.CmdHeartbeat)
                return object;
            return new $root.usb_control.CmdHeartbeat();
        };

        /**
         * Creates a plain object from a CmdHeartbeat message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.CmdHeartbeat
         * @static
         * @param {usb_control.CmdHeartbeat} message CmdHeartbeat
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CmdHeartbeat.toObject = function toObject() {
            return {};
        };

        /**
         * Converts this CmdHeartbeat to JSON.
         * @function toJSON
         * @memberof usb_control.CmdHeartbeat
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CmdHeartbeat.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CmdHeartbeat
         * @function getTypeUrl
         * @memberof usb_control.CmdHeartbeat
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CmdHeartbeat.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.CmdHeartbeat";
        };

        return CmdHeartbeat;
    })();

    usb_control.RspHeartbeat = (function() {

        /**
         * Properties of a RspHeartbeat.
         * @memberof usb_control
         * @interface IRspHeartbeat
         */

        /**
         * Constructs a new RspHeartbeat.
         * @memberof usb_control
         * @classdesc Represents a RspHeartbeat.
         * @implements IRspHeartbeat
         * @constructor
         * @param {usb_control.IRspHeartbeat=} [properties] Properties to set
         */
        function RspHeartbeat(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null && keys[i] !== "__proto__")
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * Encodes the specified RspHeartbeat message. Does not implicitly {@link usb_control.RspHeartbeat.verify|verify} messages.
         * @function encode
         * @memberof usb_control.RspHeartbeat
         * @static
         * @param {usb_control.IRspHeartbeat} message RspHeartbeat message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspHeartbeat.encode = function encode(message, writer, q) {
            if (!writer)
                writer = $Writer.create();
            if (q === undefined)
                q = 0;
            if (q > $util.recursionLimit)
                throw Error("max depth exceeded");
            return writer;
        };

        /**
         * Encodes the specified RspHeartbeat message, length delimited. Does not implicitly {@link usb_control.RspHeartbeat.verify|verify} messages.
         * @function encodeDelimited
         * @memberof usb_control.RspHeartbeat
         * @static
         * @param {usb_control.IRspHeartbeat} message RspHeartbeat message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RspHeartbeat.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer && writer.len ? writer.fork() : writer).ldelim();
        };

        /**
         * Decodes a RspHeartbeat message from the specified reader or buffer.
         * @function decode
         * @memberof usb_control.RspHeartbeat
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {usb_control.RspHeartbeat} RspHeartbeat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspHeartbeat.decode = function decode(reader, length, error, long) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            if (long === undefined)
                long = 0;
            if (long > $Reader.recursionLimit)
                throw Error("maximum nesting depth exceeded");
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.usb_control.RspHeartbeat();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                default:
                    reader.skipType(tag & 7, long);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RspHeartbeat message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof usb_control.RspHeartbeat
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {usb_control.RspHeartbeat} RspHeartbeat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RspHeartbeat.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Creates a RspHeartbeat message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof usb_control.RspHeartbeat
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {usb_control.RspHeartbeat} RspHeartbeat
         */
        RspHeartbeat.fromObject = function fromObject(object, long) {
            if (object instanceof $root.usb_control.RspHeartbeat)
                return object;
            return new $root.usb_control.RspHeartbeat();
        };

        /**
         * Creates a plain object from a RspHeartbeat message. Also converts values to other types if specified.
         * @function toObject
         * @memberof usb_control.RspHeartbeat
         * @static
         * @param {usb_control.RspHeartbeat} message RspHeartbeat
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RspHeartbeat.toObject = function toObject() {
            return {};
        };

        /**
         * Converts this RspHeartbeat to JSON.
         * @function toJSON
         * @memberof usb_control.RspHeartbeat
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RspHeartbeat.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RspHeartbeat
         * @function getTypeUrl
         * @memberof usb_control.RspHeartbeat
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RspHeartbeat.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/usb_control.RspHeartbeat";
        };

        return RspHeartbeat;
    })();

    return usb_control;
})();

export { $root as default };
