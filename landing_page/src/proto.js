/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
(function(global, factory) {
    /* global define, require, module */

    /* AMD */ if (typeof define === "function" && define.amd) {
        define(["protobufjs/minimal"], factory);
    } /* CommonJS */ else if (typeof require === "function" && typeof module === "object" && module && module.exports) {
        module.exports = factory(require("protobufjs/minimal"));
    }
})(this, function($protobuf) {
    "use strict";

    // Common aliases
    var $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

    // Exported root namespace
    var $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});

    $root.rendezvous = (function() {
        /**
         * Namespace rendezvous.
         * @exports rendezvous
         * @namespace
         */
        var rendezvous = {};

        rendezvous.pb = (function() {
            /**
             * Namespace pb.
             * @memberof rendezvous
             * @namespace
             */
            var pb = {};

            pb.Message = (function() {
                /**
                 * Properties of a Message.
                 * @memberof rendezvous.pb
                 * @interface IMessage
                 * @property {rendezvous.pb.Message.MessageType|null} [type] Message type
                 * @property {rendezvous.pb.Message.IRegister|null} [register] Message register
                 * @property {rendezvous.pb.Message.IRegisterResponse|null} [registerResponse] Message registerResponse
                 * @property {rendezvous.pb.Message.IUnregister|null} [unregister] Message unregister
                 * @property {rendezvous.pb.Message.IDiscover|null} [discover] Message discover
                 * @property {rendezvous.pb.Message.IDiscoverResponse|null} [discoverResponse] Message discoverResponse
                 */

                /**
                 * Constructs a new Message.
                 * @memberof rendezvous.pb
                 * @classdesc Represents a Message.
                 * @implements IMessage
                 * @constructor
                 * @param {rendezvous.pb.IMessage=} [properties] Properties to set
                 */
                function Message(properties) {
                    if (properties) {
                        for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i) {
                            if (properties[keys[i]] != null) {
                                this[keys[i]] = properties[keys[i]];
                            }
                        }
                    }
                }

                /**
                 * Message type.
                 * @member {rendezvous.pb.Message.MessageType} type
                 * @memberof rendezvous.pb.Message
                 * @instance
                 */
                Message.prototype.type = 0;

                /**
                 * Message register.
                 * @member {rendezvous.pb.Message.IRegister|null|undefined} register
                 * @memberof rendezvous.pb.Message
                 * @instance
                 */
                Message.prototype.register = null;

                /**
                 * Message registerResponse.
                 * @member {rendezvous.pb.Message.IRegisterResponse|null|undefined} registerResponse
                 * @memberof rendezvous.pb.Message
                 * @instance
                 */
                Message.prototype.registerResponse = null;

                /**
                 * Message unregister.
                 * @member {rendezvous.pb.Message.IUnregister|null|undefined} unregister
                 * @memberof rendezvous.pb.Message
                 * @instance
                 */
                Message.prototype.unregister = null;

                /**
                 * Message discover.
                 * @member {rendezvous.pb.Message.IDiscover|null|undefined} discover
                 * @memberof rendezvous.pb.Message
                 * @instance
                 */
                Message.prototype.discover = null;

                /**
                 * Message discoverResponse.
                 * @member {rendezvous.pb.Message.IDiscoverResponse|null|undefined} discoverResponse
                 * @memberof rendezvous.pb.Message
                 * @instance
                 */
                Message.prototype.discoverResponse = null;

                /**
                 * Creates a new Message instance using the specified properties.
                 * @function create
                 * @memberof rendezvous.pb.Message
                 * @static
                 * @param {rendezvous.pb.IMessage=} [properties] Properties to set
                 * @returns {rendezvous.pb.Message} Message instance
                 */
                Message.create = function create(properties) {
                    return new Message(properties);
                };

                /**
                 * Encodes the specified Message message. Does not implicitly {@link rendezvous.pb.Message.verify|verify} messages.
                 * @function encode
                 * @memberof rendezvous.pb.Message
                 * @static
                 * @param {rendezvous.pb.IMessage} message Message message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Message.encode = function encode(message, writer) {
                    if (!writer) {
                        writer = $Writer.create();
                    }
                    if (message.type != null && Object.hasOwnProperty.call(message, "type")) {
                        writer.uint32(/* id 1, wireType 0 =*/ 8).int32(message.type);
                    }
                    if (message.register != null && Object.hasOwnProperty.call(message, "register")) {
                        $root.rendezvous.pb.Message.Register.encode(
                            message.register,
                            writer.uint32(/* id 2, wireType 2 =*/ 18).fork(),
                        ).ldelim();
                    }
                    if (message.registerResponse != null && Object.hasOwnProperty.call(message, "registerResponse")) {
                        $root.rendezvous.pb.Message.RegisterResponse.encode(
                            message.registerResponse,
                            writer.uint32(/* id 3, wireType 2 =*/ 26).fork(),
                        ).ldelim();
                    }
                    if (message.unregister != null && Object.hasOwnProperty.call(message, "unregister")) {
                        $root.rendezvous.pb.Message.Unregister.encode(
                            message.unregister,
                            writer.uint32(/* id 4, wireType 2 =*/ 34).fork(),
                        ).ldelim();
                    }
                    if (message.discover != null && Object.hasOwnProperty.call(message, "discover")) {
                        $root.rendezvous.pb.Message.Discover.encode(
                            message.discover,
                            writer.uint32(/* id 5, wireType 2 =*/ 42).fork(),
                        ).ldelim();
                    }
                    if (message.discoverResponse != null && Object.hasOwnProperty.call(message, "discoverResponse")) {
                        $root.rendezvous.pb.Message.DiscoverResponse.encode(
                            message.discoverResponse,
                            writer.uint32(/* id 6, wireType 2 =*/ 50).fork(),
                        ).ldelim();
                    }
                    return writer;
                };

                /**
                 * Encodes the specified Message message, length delimited. Does not implicitly {@link rendezvous.pb.Message.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof rendezvous.pb.Message
                 * @static
                 * @param {rendezvous.pb.IMessage} message Message message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Message.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a Message message from the specified reader or buffer.
                 * @function decode
                 * @memberof rendezvous.pb.Message
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {rendezvous.pb.Message} Message
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Message.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader)) {
                        reader = $Reader.create(reader);
                    }
                    var end = length === undefined ? reader.len : reader.pos + length,
                        message = new $root.rendezvous.pb.Message();
                    while (reader.pos < end) {
                        var tag = reader.uint32();
                        switch (tag >>> 3) {
                            case 1:
                                message.type = reader.int32();
                                break;
                            case 2:
                                message.register = $root.rendezvous.pb.Message.Register.decode(reader, reader.uint32());
                                break;
                            case 3:
                                message.registerResponse = $root.rendezvous.pb.Message.RegisterResponse.decode(
                                    reader,
                                    reader.uint32(),
                                );
                                break;
                            case 4:
                                message.unregister = $root.rendezvous.pb.Message.Unregister.decode(
                                    reader,
                                    reader.uint32(),
                                );
                                break;
                            case 5:
                                message.discover = $root.rendezvous.pb.Message.Discover.decode(reader, reader.uint32());
                                break;
                            case 6:
                                message.discoverResponse = $root.rendezvous.pb.Message.DiscoverResponse.decode(
                                    reader,
                                    reader.uint32(),
                                );
                                break;
                            default:
                                reader.skipType(tag & 7);
                                break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a Message message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof rendezvous.pb.Message
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {rendezvous.pb.Message} Message
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Message.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader)) {
                        reader = new $Reader(reader);
                    }
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a Message message.
                 * @function verify
                 * @memberof rendezvous.pb.Message
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Message.verify = function verify(message) {
                    if (typeof message !== "object" || message === null) {
                        return "object expected";
                    }
                    if (message.type != null && message.hasOwnProperty("type")) {
                        switch (message.type) {
                            default:
                                return "type: enum value expected";
                            case 0:
                            case 1:
                            case 2:
                            case 3:
                            case 4:
                                break;
                        }
                    }
                    if (message.register != null && message.hasOwnProperty("register")) {
                        var error = $root.rendezvous.pb.Message.Register.verify(message.register);
                        if (error) {
                            return "register." + error;
                        }
                    }
                    if (message.registerResponse != null && message.hasOwnProperty("registerResponse")) {
                        var error = $root.rendezvous.pb.Message.RegisterResponse.verify(message.registerResponse);
                        if (error) {
                            return "registerResponse." + error;
                        }
                    }
                    if (message.unregister != null && message.hasOwnProperty("unregister")) {
                        var error = $root.rendezvous.pb.Message.Unregister.verify(message.unregister);
                        if (error) {
                            return "unregister." + error;
                        }
                    }
                    if (message.discover != null && message.hasOwnProperty("discover")) {
                        var error = $root.rendezvous.pb.Message.Discover.verify(message.discover);
                        if (error) {
                            return "discover." + error;
                        }
                    }
                    if (message.discoverResponse != null && message.hasOwnProperty("discoverResponse")) {
                        var error = $root.rendezvous.pb.Message.DiscoverResponse.verify(message.discoverResponse);
                        if (error) {
                            return "discoverResponse." + error;
                        }
                    }
                    return null;
                };

                /**
                 * Creates a Message message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof rendezvous.pb.Message
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {rendezvous.pb.Message} Message
                 */
                Message.fromObject = function fromObject(object) {
                    if (object instanceof $root.rendezvous.pb.Message) {
                        return object;
                    }
                    var message = new $root.rendezvous.pb.Message();
                    switch (object.type) {
                        case "REGISTER":
                        case 0:
                            message.type = 0;
                            break;
                        case "REGISTER_RESPONSE":
                        case 1:
                            message.type = 1;
                            break;
                        case "UNREGISTER":
                        case 2:
                            message.type = 2;
                            break;
                        case "DISCOVER":
                        case 3:
                            message.type = 3;
                            break;
                        case "DISCOVER_RESPONSE":
                        case 4:
                            message.type = 4;
                            break;
                    }
                    if (object.register != null) {
                        if (typeof object.register !== "object") {
                            throw TypeError(".rendezvous.pb.Message.register: object expected");
                        }
                        message.register = $root.rendezvous.pb.Message.Register.fromObject(object.register);
                    }
                    if (object.registerResponse != null) {
                        if (typeof object.registerResponse !== "object") {
                            throw TypeError(".rendezvous.pb.Message.registerResponse: object expected");
                        }
                        message.registerResponse = $root.rendezvous.pb.Message.RegisterResponse.fromObject(
                            object.registerResponse,
                        );
                    }
                    if (object.unregister != null) {
                        if (typeof object.unregister !== "object") {
                            throw TypeError(".rendezvous.pb.Message.unregister: object expected");
                        }
                        message.unregister = $root.rendezvous.pb.Message.Unregister.fromObject(object.unregister);
                    }
                    if (object.discover != null) {
                        if (typeof object.discover !== "object") {
                            throw TypeError(".rendezvous.pb.Message.discover: object expected");
                        }
                        message.discover = $root.rendezvous.pb.Message.Discover.fromObject(object.discover);
                    }
                    if (object.discoverResponse != null) {
                        if (typeof object.discoverResponse !== "object") {
                            throw TypeError(".rendezvous.pb.Message.discoverResponse: object expected");
                        }
                        message.discoverResponse = $root.rendezvous.pb.Message.DiscoverResponse.fromObject(
                            object.discoverResponse,
                        );
                    }
                    return message;
                };

                /**
                 * Creates a plain object from a Message message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof rendezvous.pb.Message
                 * @static
                 * @param {rendezvous.pb.Message} message Message
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Message.toObject = function toObject(message, options) {
                    if (!options) {
                        options = {};
                    }
                    var object = {};
                    if (options.defaults) {
                        object.type = options.enums === String ? "REGISTER" : 0;
                        object.register = null;
                        object.registerResponse = null;
                        object.unregister = null;
                        object.discover = null;
                        object.discoverResponse = null;
                    }
                    if (message.type != null && message.hasOwnProperty("type")) {
                        object.type = options.enums === String
                            ? $root.rendezvous.pb.Message.MessageType[message.type]
                            : message.type;
                    }
                    if (message.register != null && message.hasOwnProperty("register")) {
                        object.register = $root.rendezvous.pb.Message.Register.toObject(message.register, options);
                    }
                    if (message.registerResponse != null && message.hasOwnProperty("registerResponse")) {
                        object.registerResponse = $root.rendezvous.pb.Message.RegisterResponse.toObject(
                            message.registerResponse,
                            options,
                        );
                    }
                    if (message.unregister != null && message.hasOwnProperty("unregister")) {
                        object.unregister = $root.rendezvous.pb.Message.Unregister.toObject(
                            message.unregister,
                            options,
                        );
                    }
                    if (message.discover != null && message.hasOwnProperty("discover")) {
                        object.discover = $root.rendezvous.pb.Message.Discover.toObject(message.discover, options);
                    }
                    if (message.discoverResponse != null && message.hasOwnProperty("discoverResponse")) {
                        object.discoverResponse = $root.rendezvous.pb.Message.DiscoverResponse.toObject(
                            message.discoverResponse,
                            options,
                        );
                    }
                    return object;
                };

                /**
                 * Converts this Message to JSON.
                 * @function toJSON
                 * @memberof rendezvous.pb.Message
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Message.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * MessageType enum.
                 * @name rendezvous.pb.Message.MessageType
                 * @enum {number}
                 * @property {number} REGISTER=0 REGISTER value
                 * @property {number} REGISTER_RESPONSE=1 REGISTER_RESPONSE value
                 * @property {number} UNREGISTER=2 UNREGISTER value
                 * @property {number} DISCOVER=3 DISCOVER value
                 * @property {number} DISCOVER_RESPONSE=4 DISCOVER_RESPONSE value
                 */
                Message.MessageType = (function() {
                    var valuesById = {}, values = Object.create(valuesById);
                    values[valuesById[0] = "REGISTER"] = 0;
                    values[valuesById[1] = "REGISTER_RESPONSE"] = 1;
                    values[valuesById[2] = "UNREGISTER"] = 2;
                    values[valuesById[3] = "DISCOVER"] = 3;
                    values[valuesById[4] = "DISCOVER_RESPONSE"] = 4;
                    return values;
                })();

                /**
                 * ResponseStatus enum.
                 * @name rendezvous.pb.Message.ResponseStatus
                 * @enum {number}
                 * @property {number} OK=0 OK value
                 * @property {number} E_INVALID_NAMESPACE=100 E_INVALID_NAMESPACE value
                 * @property {number} E_INVALID_SIGNED_PEER_RECORD=101 E_INVALID_SIGNED_PEER_RECORD value
                 * @property {number} E_INVALID_TTL=102 E_INVALID_TTL value
                 * @property {number} E_INVALID_COOKIE=103 E_INVALID_COOKIE value
                 * @property {number} E_NOT_AUTHORIZED=200 E_NOT_AUTHORIZED value
                 * @property {number} E_INTERNAL_ERROR=300 E_INTERNAL_ERROR value
                 * @property {number} E_UNAVAILABLE=400 E_UNAVAILABLE value
                 */
                Message.ResponseStatus = (function() {
                    var valuesById = {}, values = Object.create(valuesById);
                    values[valuesById[0] = "OK"] = 0;
                    values[valuesById[100] = "E_INVALID_NAMESPACE"] = 100;
                    values[valuesById[101] = "E_INVALID_SIGNED_PEER_RECORD"] = 101;
                    values[valuesById[102] = "E_INVALID_TTL"] = 102;
                    values[valuesById[103] = "E_INVALID_COOKIE"] = 103;
                    values[valuesById[200] = "E_NOT_AUTHORIZED"] = 200;
                    values[valuesById[300] = "E_INTERNAL_ERROR"] = 300;
                    values[valuesById[400] = "E_UNAVAILABLE"] = 400;
                    return values;
                })();

                Message.Register = (function() {
                    /**
                     * Properties of a Register.
                     * @memberof rendezvous.pb.Message
                     * @interface IRegister
                     * @property {string|null} [ns] Register ns
                     * @property {Uint8Array|null} [signedPeerRecord] Register signedPeerRecord
                     * @property {number|Long|null} [ttl] Register ttl
                     */

                    /**
                     * Constructs a new Register.
                     * @memberof rendezvous.pb.Message
                     * @classdesc Represents a Register.
                     * @implements IRegister
                     * @constructor
                     * @param {rendezvous.pb.Message.IRegister=} [properties] Properties to set
                     */
                    function Register(properties) {
                        if (properties) {
                            for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i) {
                                if (properties[keys[i]] != null) {
                                    this[keys[i]] = properties[keys[i]];
                                }
                            }
                        }
                    }

                    /**
                     * Register ns.
                     * @member {string} ns
                     * @memberof rendezvous.pb.Message.Register
                     * @instance
                     */
                    Register.prototype.ns = "";

                    /**
                     * Register signedPeerRecord.
                     * @member {Uint8Array} signedPeerRecord
                     * @memberof rendezvous.pb.Message.Register
                     * @instance
                     */
                    Register.prototype.signedPeerRecord = $util.newBuffer([]);

                    /**
                     * Register ttl.
                     * @member {number|Long} ttl
                     * @memberof rendezvous.pb.Message.Register
                     * @instance
                     */
                    Register.prototype.ttl = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;

                    /**
                     * Creates a new Register instance using the specified properties.
                     * @function create
                     * @memberof rendezvous.pb.Message.Register
                     * @static
                     * @param {rendezvous.pb.Message.IRegister=} [properties] Properties to set
                     * @returns {rendezvous.pb.Message.Register} Register instance
                     */
                    Register.create = function create(properties) {
                        return new Register(properties);
                    };

                    /**
                     * Encodes the specified Register message. Does not implicitly {@link rendezvous.pb.Message.Register.verify|verify} messages.
                     * @function encode
                     * @memberof rendezvous.pb.Message.Register
                     * @static
                     * @param {rendezvous.pb.Message.IRegister} message Register message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    Register.encode = function encode(message, writer) {
                        if (!writer) {
                            writer = $Writer.create();
                        }
                        if (message.ns != null && Object.hasOwnProperty.call(message, "ns")) {
                            writer.uint32(/* id 1, wireType 2 =*/ 10).string(message.ns);
                        }
                        if (
                            message.signedPeerRecord != null && Object.hasOwnProperty.call(message, "signedPeerRecord")
                        ) {
                            writer.uint32(/* id 2, wireType 2 =*/ 18).bytes(message.signedPeerRecord);
                        }
                        if (message.ttl != null && Object.hasOwnProperty.call(message, "ttl")) {
                            writer.uint32(/* id 3, wireType 0 =*/ 24).uint64(message.ttl);
                        }
                        return writer;
                    };

                    /**
                     * Encodes the specified Register message, length delimited. Does not implicitly {@link rendezvous.pb.Message.Register.verify|verify} messages.
                     * @function encodeDelimited
                     * @memberof rendezvous.pb.Message.Register
                     * @static
                     * @param {rendezvous.pb.Message.IRegister} message Register message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    Register.encodeDelimited = function encodeDelimited(message, writer) {
                        return this.encode(message, writer).ldelim();
                    };

                    /**
                     * Decodes a Register message from the specified reader or buffer.
                     * @function decode
                     * @memberof rendezvous.pb.Message.Register
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @param {number} [length] Message length if known beforehand
                     * @returns {rendezvous.pb.Message.Register} Register
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    Register.decode = function decode(reader, length) {
                        if (!(reader instanceof $Reader)) {
                            reader = $Reader.create(reader);
                        }
                        var end = length === undefined ? reader.len : reader.pos + length,
                            message = new $root.rendezvous.pb.Message.Register();
                        while (reader.pos < end) {
                            var tag = reader.uint32();
                            switch (tag >>> 3) {
                                case 1:
                                    message.ns = reader.string();
                                    break;
                                case 2:
                                    message.signedPeerRecord = reader.bytes();
                                    break;
                                case 3:
                                    message.ttl = reader.uint64();
                                    break;
                                default:
                                    reader.skipType(tag & 7);
                                    break;
                            }
                        }
                        return message;
                    };

                    /**
                     * Decodes a Register message from the specified reader or buffer, length delimited.
                     * @function decodeDelimited
                     * @memberof rendezvous.pb.Message.Register
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @returns {rendezvous.pb.Message.Register} Register
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    Register.decodeDelimited = function decodeDelimited(reader) {
                        if (!(reader instanceof $Reader)) {
                            reader = new $Reader(reader);
                        }
                        return this.decode(reader, reader.uint32());
                    };

                    /**
                     * Verifies a Register message.
                     * @function verify
                     * @memberof rendezvous.pb.Message.Register
                     * @static
                     * @param {Object.<string,*>} message Plain object to verify
                     * @returns {string|null} `null` if valid, otherwise the reason why it is not
                     */
                    Register.verify = function verify(message) {
                        if (typeof message !== "object" || message === null) {
                            return "object expected";
                        }
                        if (message.ns != null && message.hasOwnProperty("ns")) {
                            if (!$util.isString(message.ns)) {
                                return "ns: string expected";
                            }
                        }
                        if (message.signedPeerRecord != null && message.hasOwnProperty("signedPeerRecord")) {
                            if (
                                !(message.signedPeerRecord && typeof message.signedPeerRecord.length === "number"
                                    || $util.isString(message.signedPeerRecord))
                            ) {
                                return "signedPeerRecord: buffer expected";
                            }
                        }
                        if (message.ttl != null && message.hasOwnProperty("ttl")) {
                            if (
                                !$util.isInteger(message.ttl)
                                && !(message.ttl && $util.isInteger(message.ttl.low)
                                    && $util.isInteger(message.ttl.high))
                            ) {
                                return "ttl: integer|Long expected";
                            }
                        }
                        return null;
                    };

                    /**
                     * Creates a Register message from a plain object. Also converts values to their respective internal types.
                     * @function fromObject
                     * @memberof rendezvous.pb.Message.Register
                     * @static
                     * @param {Object.<string,*>} object Plain object
                     * @returns {rendezvous.pb.Message.Register} Register
                     */
                    Register.fromObject = function fromObject(object) {
                        if (object instanceof $root.rendezvous.pb.Message.Register) {
                            return object;
                        }
                        var message = new $root.rendezvous.pb.Message.Register();
                        if (object.ns != null) {
                            message.ns = String(object.ns);
                        }
                        if (object.signedPeerRecord != null) {
                            if (typeof object.signedPeerRecord === "string") {
                                $util.base64.decode(
                                    object.signedPeerRecord,
                                    message.signedPeerRecord = $util.newBuffer(
                                        $util.base64.length(object.signedPeerRecord),
                                    ),
                                    0,
                                );
                            } else if (object.signedPeerRecord.length) {
                                message.signedPeerRecord = object.signedPeerRecord;
                            }
                        }
                        if (object.ttl != null) {
                            if ($util.Long) {
                                (message.ttl = $util.Long.fromValue(object.ttl)).unsigned = true;
                            } else if (typeof object.ttl === "string") {
                                message.ttl = parseInt(object.ttl, 10);
                            } else if (typeof object.ttl === "number") {
                                message.ttl = object.ttl;
                            } else if (typeof object.ttl === "object") {
                                message.ttl = new $util.LongBits(object.ttl.low >>> 0, object.ttl.high >>> 0).toNumber(
                                    true,
                                );
                            }
                        }
                        return message;
                    };

                    /**
                     * Creates a plain object from a Register message. Also converts values to other types if specified.
                     * @function toObject
                     * @memberof rendezvous.pb.Message.Register
                     * @static
                     * @param {rendezvous.pb.Message.Register} message Register
                     * @param {$protobuf.IConversionOptions} [options] Conversion options
                     * @returns {Object.<string,*>} Plain object
                     */
                    Register.toObject = function toObject(message, options) {
                        if (!options) {
                            options = {};
                        }
                        var object = {};
                        if (options.defaults) {
                            object.ns = "";
                            if (options.bytes === String) {
                                object.signedPeerRecord = "";
                            } else {
                                object.signedPeerRecord = [];
                                if (options.bytes !== Array) {
                                    object.signedPeerRecord = $util.newBuffer(object.signedPeerRecord);
                                }
                            }
                            if ($util.Long) {
                                var long = new $util.Long(0, 0, true);
                                object.ttl = options.longs === String
                                    ? long.toString()
                                    : options.longs === Number
                                    ? long.toNumber()
                                    : long;
                            } else {
                                object.ttl = options.longs === String ? "0" : 0;
                            }
                        }
                        if (message.ns != null && message.hasOwnProperty("ns")) {
                            object.ns = message.ns;
                        }
                        if (message.signedPeerRecord != null && message.hasOwnProperty("signedPeerRecord")) {
                            object.signedPeerRecord = options.bytes === String
                                ? $util.base64.encode(message.signedPeerRecord, 0, message.signedPeerRecord.length)
                                : options.bytes === Array
                                ? Array.prototype.slice.call(message.signedPeerRecord)
                                : message.signedPeerRecord;
                        }
                        if (message.ttl != null && message.hasOwnProperty("ttl")) {
                            if (typeof message.ttl === "number") {
                                object.ttl = options.longs === String ? String(message.ttl) : message.ttl;
                            } else {
                                object.ttl = options.longs === String
                                    ? $util.Long.prototype.toString.call(message.ttl)
                                    : options.longs === Number
                                    ? new $util.LongBits(message.ttl.low >>> 0, message.ttl.high >>> 0).toNumber(true)
                                    : message.ttl;
                            }
                        }
                        return object;
                    };

                    /**
                     * Converts this Register to JSON.
                     * @function toJSON
                     * @memberof rendezvous.pb.Message.Register
                     * @instance
                     * @returns {Object.<string,*>} JSON object
                     */
                    Register.prototype.toJSON = function toJSON() {
                        return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                    };

                    return Register;
                })();

                Message.RegisterResponse = (function() {
                    /**
                     * Properties of a RegisterResponse.
                     * @memberof rendezvous.pb.Message
                     * @interface IRegisterResponse
                     * @property {rendezvous.pb.Message.ResponseStatus|null} [status] RegisterResponse status
                     * @property {string|null} [statusText] RegisterResponse statusText
                     * @property {number|Long|null} [ttl] RegisterResponse ttl
                     */

                    /**
                     * Constructs a new RegisterResponse.
                     * @memberof rendezvous.pb.Message
                     * @classdesc Represents a RegisterResponse.
                     * @implements IRegisterResponse
                     * @constructor
                     * @param {rendezvous.pb.Message.IRegisterResponse=} [properties] Properties to set
                     */
                    function RegisterResponse(properties) {
                        if (properties) {
                            for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i) {
                                if (properties[keys[i]] != null) {
                                    this[keys[i]] = properties[keys[i]];
                                }
                            }
                        }
                    }

                    /**
                     * RegisterResponse status.
                     * @member {rendezvous.pb.Message.ResponseStatus} status
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @instance
                     */
                    RegisterResponse.prototype.status = 0;

                    /**
                     * RegisterResponse statusText.
                     * @member {string} statusText
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @instance
                     */
                    RegisterResponse.prototype.statusText = "";

                    /**
                     * RegisterResponse ttl.
                     * @member {number|Long} ttl
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @instance
                     */
                    RegisterResponse.prototype.ttl = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;

                    /**
                     * Creates a new RegisterResponse instance using the specified properties.
                     * @function create
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @static
                     * @param {rendezvous.pb.Message.IRegisterResponse=} [properties] Properties to set
                     * @returns {rendezvous.pb.Message.RegisterResponse} RegisterResponse instance
                     */
                    RegisterResponse.create = function create(properties) {
                        return new RegisterResponse(properties);
                    };

                    /**
                     * Encodes the specified RegisterResponse message. Does not implicitly {@link rendezvous.pb.Message.RegisterResponse.verify|verify} messages.
                     * @function encode
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @static
                     * @param {rendezvous.pb.Message.IRegisterResponse} message RegisterResponse message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    RegisterResponse.encode = function encode(message, writer) {
                        if (!writer) {
                            writer = $Writer.create();
                        }
                        if (message.status != null && Object.hasOwnProperty.call(message, "status")) {
                            writer.uint32(/* id 1, wireType 0 =*/ 8).int32(message.status);
                        }
                        if (message.statusText != null && Object.hasOwnProperty.call(message, "statusText")) {
                            writer.uint32(/* id 2, wireType 2 =*/ 18).string(message.statusText);
                        }
                        if (message.ttl != null && Object.hasOwnProperty.call(message, "ttl")) {
                            writer.uint32(/* id 3, wireType 0 =*/ 24).uint64(message.ttl);
                        }
                        return writer;
                    };

                    /**
                     * Encodes the specified RegisterResponse message, length delimited. Does not implicitly {@link rendezvous.pb.Message.RegisterResponse.verify|verify} messages.
                     * @function encodeDelimited
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @static
                     * @param {rendezvous.pb.Message.IRegisterResponse} message RegisterResponse message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    RegisterResponse.encodeDelimited = function encodeDelimited(message, writer) {
                        return this.encode(message, writer).ldelim();
                    };

                    /**
                     * Decodes a RegisterResponse message from the specified reader or buffer.
                     * @function decode
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @param {number} [length] Message length if known beforehand
                     * @returns {rendezvous.pb.Message.RegisterResponse} RegisterResponse
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    RegisterResponse.decode = function decode(reader, length) {
                        if (!(reader instanceof $Reader)) {
                            reader = $Reader.create(reader);
                        }
                        var end = length === undefined ? reader.len : reader.pos + length,
                            message = new $root.rendezvous.pb.Message.RegisterResponse();
                        while (reader.pos < end) {
                            var tag = reader.uint32();
                            switch (tag >>> 3) {
                                case 1:
                                    message.status = reader.int32();
                                    break;
                                case 2:
                                    message.statusText = reader.string();
                                    break;
                                case 3:
                                    message.ttl = reader.uint64();
                                    break;
                                default:
                                    reader.skipType(tag & 7);
                                    break;
                            }
                        }
                        return message;
                    };

                    /**
                     * Decodes a RegisterResponse message from the specified reader or buffer, length delimited.
                     * @function decodeDelimited
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @returns {rendezvous.pb.Message.RegisterResponse} RegisterResponse
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    RegisterResponse.decodeDelimited = function decodeDelimited(reader) {
                        if (!(reader instanceof $Reader)) {
                            reader = new $Reader(reader);
                        }
                        return this.decode(reader, reader.uint32());
                    };

                    /**
                     * Verifies a RegisterResponse message.
                     * @function verify
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @static
                     * @param {Object.<string,*>} message Plain object to verify
                     * @returns {string|null} `null` if valid, otherwise the reason why it is not
                     */
                    RegisterResponse.verify = function verify(message) {
                        if (typeof message !== "object" || message === null) {
                            return "object expected";
                        }
                        if (message.status != null && message.hasOwnProperty("status")) {
                            switch (message.status) {
                                default:
                                    return "status: enum value expected";
                                case 0:
                                case 100:
                                case 101:
                                case 102:
                                case 103:
                                case 200:
                                case 300:
                                case 400:
                                    break;
                            }
                        }
                        if (message.statusText != null && message.hasOwnProperty("statusText")) {
                            if (!$util.isString(message.statusText)) {
                                return "statusText: string expected";
                            }
                        }
                        if (message.ttl != null && message.hasOwnProperty("ttl")) {
                            if (
                                !$util.isInteger(message.ttl)
                                && !(message.ttl && $util.isInteger(message.ttl.low)
                                    && $util.isInteger(message.ttl.high))
                            ) {
                                return "ttl: integer|Long expected";
                            }
                        }
                        return null;
                    };

                    /**
                     * Creates a RegisterResponse message from a plain object. Also converts values to their respective internal types.
                     * @function fromObject
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @static
                     * @param {Object.<string,*>} object Plain object
                     * @returns {rendezvous.pb.Message.RegisterResponse} RegisterResponse
                     */
                    RegisterResponse.fromObject = function fromObject(object) {
                        if (object instanceof $root.rendezvous.pb.Message.RegisterResponse) {
                            return object;
                        }
                        var message = new $root.rendezvous.pb.Message.RegisterResponse();
                        switch (object.status) {
                            case "OK":
                            case 0:
                                message.status = 0;
                                break;
                            case "E_INVALID_NAMESPACE":
                            case 100:
                                message.status = 100;
                                break;
                            case "E_INVALID_SIGNED_PEER_RECORD":
                            case 101:
                                message.status = 101;
                                break;
                            case "E_INVALID_TTL":
                            case 102:
                                message.status = 102;
                                break;
                            case "E_INVALID_COOKIE":
                            case 103:
                                message.status = 103;
                                break;
                            case "E_NOT_AUTHORIZED":
                            case 200:
                                message.status = 200;
                                break;
                            case "E_INTERNAL_ERROR":
                            case 300:
                                message.status = 300;
                                break;
                            case "E_UNAVAILABLE":
                            case 400:
                                message.status = 400;
                                break;
                        }
                        if (object.statusText != null) {
                            message.statusText = String(object.statusText);
                        }
                        if (object.ttl != null) {
                            if ($util.Long) {
                                (message.ttl = $util.Long.fromValue(object.ttl)).unsigned = true;
                            } else if (typeof object.ttl === "string") {
                                message.ttl = parseInt(object.ttl, 10);
                            } else if (typeof object.ttl === "number") {
                                message.ttl = object.ttl;
                            } else if (typeof object.ttl === "object") {
                                message.ttl = new $util.LongBits(object.ttl.low >>> 0, object.ttl.high >>> 0).toNumber(
                                    true,
                                );
                            }
                        }
                        return message;
                    };

                    /**
                     * Creates a plain object from a RegisterResponse message. Also converts values to other types if specified.
                     * @function toObject
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @static
                     * @param {rendezvous.pb.Message.RegisterResponse} message RegisterResponse
                     * @param {$protobuf.IConversionOptions} [options] Conversion options
                     * @returns {Object.<string,*>} Plain object
                     */
                    RegisterResponse.toObject = function toObject(message, options) {
                        if (!options) {
                            options = {};
                        }
                        var object = {};
                        if (options.defaults) {
                            object.status = options.enums === String ? "OK" : 0;
                            object.statusText = "";
                            if ($util.Long) {
                                var long = new $util.Long(0, 0, true);
                                object.ttl = options.longs === String
                                    ? long.toString()
                                    : options.longs === Number
                                    ? long.toNumber()
                                    : long;
                            } else {
                                object.ttl = options.longs === String ? "0" : 0;
                            }
                        }
                        if (message.status != null && message.hasOwnProperty("status")) {
                            object.status = options.enums === String
                                ? $root.rendezvous.pb.Message.ResponseStatus[message.status]
                                : message.status;
                        }
                        if (message.statusText != null && message.hasOwnProperty("statusText")) {
                            object.statusText = message.statusText;
                        }
                        if (message.ttl != null && message.hasOwnProperty("ttl")) {
                            if (typeof message.ttl === "number") {
                                object.ttl = options.longs === String ? String(message.ttl) : message.ttl;
                            } else {
                                object.ttl = options.longs === String
                                    ? $util.Long.prototype.toString.call(message.ttl)
                                    : options.longs === Number
                                    ? new $util.LongBits(message.ttl.low >>> 0, message.ttl.high >>> 0).toNumber(true)
                                    : message.ttl;
                            }
                        }
                        return object;
                    };

                    /**
                     * Converts this RegisterResponse to JSON.
                     * @function toJSON
                     * @memberof rendezvous.pb.Message.RegisterResponse
                     * @instance
                     * @returns {Object.<string,*>} JSON object
                     */
                    RegisterResponse.prototype.toJSON = function toJSON() {
                        return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                    };

                    return RegisterResponse;
                })();

                Message.Unregister = (function() {
                    /**
                     * Properties of an Unregister.
                     * @memberof rendezvous.pb.Message
                     * @interface IUnregister
                     * @property {string|null} [ns] Unregister ns
                     * @property {Uint8Array|null} [id] Unregister id
                     */

                    /**
                     * Constructs a new Unregister.
                     * @memberof rendezvous.pb.Message
                     * @classdesc Represents an Unregister.
                     * @implements IUnregister
                     * @constructor
                     * @param {rendezvous.pb.Message.IUnregister=} [properties] Properties to set
                     */
                    function Unregister(properties) {
                        if (properties) {
                            for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i) {
                                if (properties[keys[i]] != null) {
                                    this[keys[i]] = properties[keys[i]];
                                }
                            }
                        }
                    }

                    /**
                     * Unregister ns.
                     * @member {string} ns
                     * @memberof rendezvous.pb.Message.Unregister
                     * @instance
                     */
                    Unregister.prototype.ns = "";

                    /**
                     * Unregister id.
                     * @member {Uint8Array} id
                     * @memberof rendezvous.pb.Message.Unregister
                     * @instance
                     */
                    Unregister.prototype.id = $util.newBuffer([]);

                    /**
                     * Creates a new Unregister instance using the specified properties.
                     * @function create
                     * @memberof rendezvous.pb.Message.Unregister
                     * @static
                     * @param {rendezvous.pb.Message.IUnregister=} [properties] Properties to set
                     * @returns {rendezvous.pb.Message.Unregister} Unregister instance
                     */
                    Unregister.create = function create(properties) {
                        return new Unregister(properties);
                    };

                    /**
                     * Encodes the specified Unregister message. Does not implicitly {@link rendezvous.pb.Message.Unregister.verify|verify} messages.
                     * @function encode
                     * @memberof rendezvous.pb.Message.Unregister
                     * @static
                     * @param {rendezvous.pb.Message.IUnregister} message Unregister message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    Unregister.encode = function encode(message, writer) {
                        if (!writer) {
                            writer = $Writer.create();
                        }
                        if (message.ns != null && Object.hasOwnProperty.call(message, "ns")) {
                            writer.uint32(/* id 1, wireType 2 =*/ 10).string(message.ns);
                        }
                        if (message.id != null && Object.hasOwnProperty.call(message, "id")) {
                            writer.uint32(/* id 2, wireType 2 =*/ 18).bytes(message.id);
                        }
                        return writer;
                    };

                    /**
                     * Encodes the specified Unregister message, length delimited. Does not implicitly {@link rendezvous.pb.Message.Unregister.verify|verify} messages.
                     * @function encodeDelimited
                     * @memberof rendezvous.pb.Message.Unregister
                     * @static
                     * @param {rendezvous.pb.Message.IUnregister} message Unregister message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    Unregister.encodeDelimited = function encodeDelimited(message, writer) {
                        return this.encode(message, writer).ldelim();
                    };

                    /**
                     * Decodes an Unregister message from the specified reader or buffer.
                     * @function decode
                     * @memberof rendezvous.pb.Message.Unregister
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @param {number} [length] Message length if known beforehand
                     * @returns {rendezvous.pb.Message.Unregister} Unregister
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    Unregister.decode = function decode(reader, length) {
                        if (!(reader instanceof $Reader)) {
                            reader = $Reader.create(reader);
                        }
                        var end = length === undefined ? reader.len : reader.pos + length,
                            message = new $root.rendezvous.pb.Message.Unregister();
                        while (reader.pos < end) {
                            var tag = reader.uint32();
                            switch (tag >>> 3) {
                                case 1:
                                    message.ns = reader.string();
                                    break;
                                case 2:
                                    message.id = reader.bytes();
                                    break;
                                default:
                                    reader.skipType(tag & 7);
                                    break;
                            }
                        }
                        return message;
                    };

                    /**
                     * Decodes an Unregister message from the specified reader or buffer, length delimited.
                     * @function decodeDelimited
                     * @memberof rendezvous.pb.Message.Unregister
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @returns {rendezvous.pb.Message.Unregister} Unregister
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    Unregister.decodeDelimited = function decodeDelimited(reader) {
                        if (!(reader instanceof $Reader)) {
                            reader = new $Reader(reader);
                        }
                        return this.decode(reader, reader.uint32());
                    };

                    /**
                     * Verifies an Unregister message.
                     * @function verify
                     * @memberof rendezvous.pb.Message.Unregister
                     * @static
                     * @param {Object.<string,*>} message Plain object to verify
                     * @returns {string|null} `null` if valid, otherwise the reason why it is not
                     */
                    Unregister.verify = function verify(message) {
                        if (typeof message !== "object" || message === null) {
                            return "object expected";
                        }
                        if (message.ns != null && message.hasOwnProperty("ns")) {
                            if (!$util.isString(message.ns)) {
                                return "ns: string expected";
                            }
                        }
                        if (message.id != null && message.hasOwnProperty("id")) {
                            if (!(message.id && typeof message.id.length === "number" || $util.isString(message.id))) {
                                return "id: buffer expected";
                            }
                        }
                        return null;
                    };

                    /**
                     * Creates an Unregister message from a plain object. Also converts values to their respective internal types.
                     * @function fromObject
                     * @memberof rendezvous.pb.Message.Unregister
                     * @static
                     * @param {Object.<string,*>} object Plain object
                     * @returns {rendezvous.pb.Message.Unregister} Unregister
                     */
                    Unregister.fromObject = function fromObject(object) {
                        if (object instanceof $root.rendezvous.pb.Message.Unregister) {
                            return object;
                        }
                        var message = new $root.rendezvous.pb.Message.Unregister();
                        if (object.ns != null) {
                            message.ns = String(object.ns);
                        }
                        if (object.id != null) {
                            if (typeof object.id === "string") {
                                $util.base64.decode(
                                    object.id,
                                    message.id = $util.newBuffer($util.base64.length(object.id)),
                                    0,
                                );
                            } else if (object.id.length) {
                                message.id = object.id;
                            }
                        }
                        return message;
                    };

                    /**
                     * Creates a plain object from an Unregister message. Also converts values to other types if specified.
                     * @function toObject
                     * @memberof rendezvous.pb.Message.Unregister
                     * @static
                     * @param {rendezvous.pb.Message.Unregister} message Unregister
                     * @param {$protobuf.IConversionOptions} [options] Conversion options
                     * @returns {Object.<string,*>} Plain object
                     */
                    Unregister.toObject = function toObject(message, options) {
                        if (!options) {
                            options = {};
                        }
                        var object = {};
                        if (options.defaults) {
                            object.ns = "";
                            if (options.bytes === String) {
                                object.id = "";
                            } else {
                                object.id = [];
                                if (options.bytes !== Array) {
                                    object.id = $util.newBuffer(object.id);
                                }
                            }
                        }
                        if (message.ns != null && message.hasOwnProperty("ns")) {
                            object.ns = message.ns;
                        }
                        if (message.id != null && message.hasOwnProperty("id")) {
                            object.id = options.bytes === String
                                ? $util.base64.encode(message.id, 0, message.id.length)
                                : options.bytes === Array
                                ? Array.prototype.slice.call(message.id)
                                : message.id;
                        }
                        return object;
                    };

                    /**
                     * Converts this Unregister to JSON.
                     * @function toJSON
                     * @memberof rendezvous.pb.Message.Unregister
                     * @instance
                     * @returns {Object.<string,*>} JSON object
                     */
                    Unregister.prototype.toJSON = function toJSON() {
                        return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                    };

                    return Unregister;
                })();

                Message.Discover = (function() {
                    /**
                     * Properties of a Discover.
                     * @memberof rendezvous.pb.Message
                     * @interface IDiscover
                     * @property {string|null} [ns] Discover ns
                     * @property {number|Long|null} [limit] Discover limit
                     * @property {Uint8Array|null} [cookie] Discover cookie
                     */

                    /**
                     * Constructs a new Discover.
                     * @memberof rendezvous.pb.Message
                     * @classdesc Represents a Discover.
                     * @implements IDiscover
                     * @constructor
                     * @param {rendezvous.pb.Message.IDiscover=} [properties] Properties to set
                     */
                    function Discover(properties) {
                        if (properties) {
                            for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i) {
                                if (properties[keys[i]] != null) {
                                    this[keys[i]] = properties[keys[i]];
                                }
                            }
                        }
                    }

                    /**
                     * Discover ns.
                     * @member {string} ns
                     * @memberof rendezvous.pb.Message.Discover
                     * @instance
                     */
                    Discover.prototype.ns = "";

                    /**
                     * Discover limit.
                     * @member {number|Long} limit
                     * @memberof rendezvous.pb.Message.Discover
                     * @instance
                     */
                    Discover.prototype.limit = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;

                    /**
                     * Discover cookie.
                     * @member {Uint8Array} cookie
                     * @memberof rendezvous.pb.Message.Discover
                     * @instance
                     */
                    Discover.prototype.cookie = $util.newBuffer([]);

                    /**
                     * Creates a new Discover instance using the specified properties.
                     * @function create
                     * @memberof rendezvous.pb.Message.Discover
                     * @static
                     * @param {rendezvous.pb.Message.IDiscover=} [properties] Properties to set
                     * @returns {rendezvous.pb.Message.Discover} Discover instance
                     */
                    Discover.create = function create(properties) {
                        return new Discover(properties);
                    };

                    /**
                     * Encodes the specified Discover message. Does not implicitly {@link rendezvous.pb.Message.Discover.verify|verify} messages.
                     * @function encode
                     * @memberof rendezvous.pb.Message.Discover
                     * @static
                     * @param {rendezvous.pb.Message.IDiscover} message Discover message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    Discover.encode = function encode(message, writer) {
                        if (!writer) {
                            writer = $Writer.create();
                        }
                        if (message.ns != null && Object.hasOwnProperty.call(message, "ns")) {
                            writer.uint32(/* id 1, wireType 2 =*/ 10).string(message.ns);
                        }
                        if (message.limit != null && Object.hasOwnProperty.call(message, "limit")) {
                            writer.uint32(/* id 2, wireType 0 =*/ 16).uint64(message.limit);
                        }
                        if (message.cookie != null && Object.hasOwnProperty.call(message, "cookie")) {
                            writer.uint32(/* id 3, wireType 2 =*/ 26).bytes(message.cookie);
                        }
                        return writer;
                    };

                    /**
                     * Encodes the specified Discover message, length delimited. Does not implicitly {@link rendezvous.pb.Message.Discover.verify|verify} messages.
                     * @function encodeDelimited
                     * @memberof rendezvous.pb.Message.Discover
                     * @static
                     * @param {rendezvous.pb.Message.IDiscover} message Discover message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    Discover.encodeDelimited = function encodeDelimited(message, writer) {
                        return this.encode(message, writer).ldelim();
                    };

                    /**
                     * Decodes a Discover message from the specified reader or buffer.
                     * @function decode
                     * @memberof rendezvous.pb.Message.Discover
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @param {number} [length] Message length if known beforehand
                     * @returns {rendezvous.pb.Message.Discover} Discover
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    Discover.decode = function decode(reader, length) {
                        if (!(reader instanceof $Reader)) {
                            reader = $Reader.create(reader);
                        }
                        var end = length === undefined ? reader.len : reader.pos + length,
                            message = new $root.rendezvous.pb.Message.Discover();
                        while (reader.pos < end) {
                            var tag = reader.uint32();
                            switch (tag >>> 3) {
                                case 1:
                                    message.ns = reader.string();
                                    break;
                                case 2:
                                    message.limit = reader.uint64();
                                    break;
                                case 3:
                                    message.cookie = reader.bytes();
                                    break;
                                default:
                                    reader.skipType(tag & 7);
                                    break;
                            }
                        }
                        return message;
                    };

                    /**
                     * Decodes a Discover message from the specified reader or buffer, length delimited.
                     * @function decodeDelimited
                     * @memberof rendezvous.pb.Message.Discover
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @returns {rendezvous.pb.Message.Discover} Discover
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    Discover.decodeDelimited = function decodeDelimited(reader) {
                        if (!(reader instanceof $Reader)) {
                            reader = new $Reader(reader);
                        }
                        return this.decode(reader, reader.uint32());
                    };

                    /**
                     * Verifies a Discover message.
                     * @function verify
                     * @memberof rendezvous.pb.Message.Discover
                     * @static
                     * @param {Object.<string,*>} message Plain object to verify
                     * @returns {string|null} `null` if valid, otherwise the reason why it is not
                     */
                    Discover.verify = function verify(message) {
                        if (typeof message !== "object" || message === null) {
                            return "object expected";
                        }
                        if (message.ns != null && message.hasOwnProperty("ns")) {
                            if (!$util.isString(message.ns)) {
                                return "ns: string expected";
                            }
                        }
                        if (message.limit != null && message.hasOwnProperty("limit")) {
                            if (
                                !$util.isInteger(message.limit)
                                && !(message.limit && $util.isInteger(message.limit.low)
                                    && $util.isInteger(message.limit.high))
                            ) {
                                return "limit: integer|Long expected";
                            }
                        }
                        if (message.cookie != null && message.hasOwnProperty("cookie")) {
                            if (
                                !(message.cookie && typeof message.cookie.length === "number"
                                    || $util.isString(message.cookie))
                            ) {
                                return "cookie: buffer expected";
                            }
                        }
                        return null;
                    };

                    /**
                     * Creates a Discover message from a plain object. Also converts values to their respective internal types.
                     * @function fromObject
                     * @memberof rendezvous.pb.Message.Discover
                     * @static
                     * @param {Object.<string,*>} object Plain object
                     * @returns {rendezvous.pb.Message.Discover} Discover
                     */
                    Discover.fromObject = function fromObject(object) {
                        if (object instanceof $root.rendezvous.pb.Message.Discover) {
                            return object;
                        }
                        var message = new $root.rendezvous.pb.Message.Discover();
                        if (object.ns != null) {
                            message.ns = String(object.ns);
                        }
                        if (object.limit != null) {
                            if ($util.Long) {
                                (message.limit = $util.Long.fromValue(object.limit)).unsigned = true;
                            } else if (typeof object.limit === "string") {
                                message.limit = parseInt(object.limit, 10);
                            } else if (typeof object.limit === "number") {
                                message.limit = object.limit;
                            } else if (typeof object.limit === "object") {
                                message.limit = new $util.LongBits(object.limit.low >>> 0, object.limit.high >>> 0)
                                    .toNumber(true);
                            }
                        }
                        if (object.cookie != null) {
                            if (typeof object.cookie === "string") {
                                $util.base64.decode(
                                    object.cookie,
                                    message.cookie = $util.newBuffer($util.base64.length(object.cookie)),
                                    0,
                                );
                            } else if (object.cookie.length) {
                                message.cookie = object.cookie;
                            }
                        }
                        return message;
                    };

                    /**
                     * Creates a plain object from a Discover message. Also converts values to other types if specified.
                     * @function toObject
                     * @memberof rendezvous.pb.Message.Discover
                     * @static
                     * @param {rendezvous.pb.Message.Discover} message Discover
                     * @param {$protobuf.IConversionOptions} [options] Conversion options
                     * @returns {Object.<string,*>} Plain object
                     */
                    Discover.toObject = function toObject(message, options) {
                        if (!options) {
                            options = {};
                        }
                        var object = {};
                        if (options.defaults) {
                            object.ns = "";
                            if ($util.Long) {
                                var long = new $util.Long(0, 0, true);
                                object.limit = options.longs === String
                                    ? long.toString()
                                    : options.longs === Number
                                    ? long.toNumber()
                                    : long;
                            } else {
                                object.limit = options.longs === String ? "0" : 0;
                            }
                            if (options.bytes === String) {
                                object.cookie = "";
                            } else {
                                object.cookie = [];
                                if (options.bytes !== Array) {
                                    object.cookie = $util.newBuffer(object.cookie);
                                }
                            }
                        }
                        if (message.ns != null && message.hasOwnProperty("ns")) {
                            object.ns = message.ns;
                        }
                        if (message.limit != null && message.hasOwnProperty("limit")) {
                            if (typeof message.limit === "number") {
                                object.limit = options.longs === String ? String(message.limit) : message.limit;
                            } else {
                                object.limit = options.longs === String
                                    ? $util.Long.prototype.toString.call(message.limit)
                                    : options.longs === Number
                                    ? new $util.LongBits(message.limit.low >>> 0, message.limit.high >>> 0).toNumber(
                                        true,
                                    )
                                    : message.limit;
                            }
                        }
                        if (message.cookie != null && message.hasOwnProperty("cookie")) {
                            object.cookie = options.bytes === String
                                ? $util.base64.encode(message.cookie, 0, message.cookie.length)
                                : options.bytes === Array
                                ? Array.prototype.slice.call(message.cookie)
                                : message.cookie;
                        }
                        return object;
                    };

                    /**
                     * Converts this Discover to JSON.
                     * @function toJSON
                     * @memberof rendezvous.pb.Message.Discover
                     * @instance
                     * @returns {Object.<string,*>} JSON object
                     */
                    Discover.prototype.toJSON = function toJSON() {
                        return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                    };

                    return Discover;
                })();

                Message.DiscoverResponse = (function() {
                    /**
                     * Properties of a DiscoverResponse.
                     * @memberof rendezvous.pb.Message
                     * @interface IDiscoverResponse
                     * @property {Array.<rendezvous.pb.Message.IRegister>|null} [registrations] DiscoverResponse registrations
                     * @property {Uint8Array|null} [cookie] DiscoverResponse cookie
                     * @property {rendezvous.pb.Message.ResponseStatus|null} [status] DiscoverResponse status
                     * @property {string|null} [statusText] DiscoverResponse statusText
                     */

                    /**
                     * Constructs a new DiscoverResponse.
                     * @memberof rendezvous.pb.Message
                     * @classdesc Represents a DiscoverResponse.
                     * @implements IDiscoverResponse
                     * @constructor
                     * @param {rendezvous.pb.Message.IDiscoverResponse=} [properties] Properties to set
                     */
                    function DiscoverResponse(properties) {
                        this.registrations = [];
                        if (properties) {
                            for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i) {
                                if (properties[keys[i]] != null) {
                                    this[keys[i]] = properties[keys[i]];
                                }
                            }
                        }
                    }

                    /**
                     * DiscoverResponse registrations.
                     * @member {Array.<rendezvous.pb.Message.IRegister>} registrations
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @instance
                     */
                    DiscoverResponse.prototype.registrations = $util.emptyArray;

                    /**
                     * DiscoverResponse cookie.
                     * @member {Uint8Array} cookie
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @instance
                     */
                    DiscoverResponse.prototype.cookie = $util.newBuffer([]);

                    /**
                     * DiscoverResponse status.
                     * @member {rendezvous.pb.Message.ResponseStatus} status
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @instance
                     */
                    DiscoverResponse.prototype.status = 0;

                    /**
                     * DiscoverResponse statusText.
                     * @member {string} statusText
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @instance
                     */
                    DiscoverResponse.prototype.statusText = "";

                    /**
                     * Creates a new DiscoverResponse instance using the specified properties.
                     * @function create
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @static
                     * @param {rendezvous.pb.Message.IDiscoverResponse=} [properties] Properties to set
                     * @returns {rendezvous.pb.Message.DiscoverResponse} DiscoverResponse instance
                     */
                    DiscoverResponse.create = function create(properties) {
                        return new DiscoverResponse(properties);
                    };

                    /**
                     * Encodes the specified DiscoverResponse message. Does not implicitly {@link rendezvous.pb.Message.DiscoverResponse.verify|verify} messages.
                     * @function encode
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @static
                     * @param {rendezvous.pb.Message.IDiscoverResponse} message DiscoverResponse message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    DiscoverResponse.encode = function encode(message, writer) {
                        if (!writer) {
                            writer = $Writer.create();
                        }
                        if (message.registrations != null && message.registrations.length) {
                            for (var i = 0; i < message.registrations.length; ++i) {
                                $root.rendezvous.pb.Message.Register.encode(
                                    message.registrations[i],
                                    writer.uint32(/* id 1, wireType 2 =*/ 10).fork(),
                                ).ldelim();
                            }
                        }
                        if (message.cookie != null && Object.hasOwnProperty.call(message, "cookie")) {
                            writer.uint32(/* id 2, wireType 2 =*/ 18).bytes(message.cookie);
                        }
                        if (message.status != null && Object.hasOwnProperty.call(message, "status")) {
                            writer.uint32(/* id 3, wireType 0 =*/ 24).int32(message.status);
                        }
                        if (message.statusText != null && Object.hasOwnProperty.call(message, "statusText")) {
                            writer.uint32(/* id 4, wireType 2 =*/ 34).string(message.statusText);
                        }
                        return writer;
                    };

                    /**
                     * Encodes the specified DiscoverResponse message, length delimited. Does not implicitly {@link rendezvous.pb.Message.DiscoverResponse.verify|verify} messages.
                     * @function encodeDelimited
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @static
                     * @param {rendezvous.pb.Message.IDiscoverResponse} message DiscoverResponse message or plain object to encode
                     * @param {$protobuf.Writer} [writer] Writer to encode to
                     * @returns {$protobuf.Writer} Writer
                     */
                    DiscoverResponse.encodeDelimited = function encodeDelimited(message, writer) {
                        return this.encode(message, writer).ldelim();
                    };

                    /**
                     * Decodes a DiscoverResponse message from the specified reader or buffer.
                     * @function decode
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @param {number} [length] Message length if known beforehand
                     * @returns {rendezvous.pb.Message.DiscoverResponse} DiscoverResponse
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    DiscoverResponse.decode = function decode(reader, length) {
                        if (!(reader instanceof $Reader)) {
                            reader = $Reader.create(reader);
                        }
                        var end = length === undefined ? reader.len : reader.pos + length,
                            message = new $root.rendezvous.pb.Message.DiscoverResponse();
                        while (reader.pos < end) {
                            var tag = reader.uint32();
                            switch (tag >>> 3) {
                                case 1:
                                    if (!(message.registrations && message.registrations.length)) {
                                        message.registrations = [];
                                    }
                                    message.registrations.push(
                                        $root.rendezvous.pb.Message.Register.decode(reader, reader.uint32()),
                                    );
                                    break;
                                case 2:
                                    message.cookie = reader.bytes();
                                    break;
                                case 3:
                                    message.status = reader.int32();
                                    break;
                                case 4:
                                    message.statusText = reader.string();
                                    break;
                                default:
                                    reader.skipType(tag & 7);
                                    break;
                            }
                        }
                        return message;
                    };

                    /**
                     * Decodes a DiscoverResponse message from the specified reader or buffer, length delimited.
                     * @function decodeDelimited
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @static
                     * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                     * @returns {rendezvous.pb.Message.DiscoverResponse} DiscoverResponse
                     * @throws {Error} If the payload is not a reader or valid buffer
                     * @throws {$protobuf.util.ProtocolError} If required fields are missing
                     */
                    DiscoverResponse.decodeDelimited = function decodeDelimited(reader) {
                        if (!(reader instanceof $Reader)) {
                            reader = new $Reader(reader);
                        }
                        return this.decode(reader, reader.uint32());
                    };

                    /**
                     * Verifies a DiscoverResponse message.
                     * @function verify
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @static
                     * @param {Object.<string,*>} message Plain object to verify
                     * @returns {string|null} `null` if valid, otherwise the reason why it is not
                     */
                    DiscoverResponse.verify = function verify(message) {
                        if (typeof message !== "object" || message === null) {
                            return "object expected";
                        }
                        if (message.registrations != null && message.hasOwnProperty("registrations")) {
                            if (!Array.isArray(message.registrations)) {
                                return "registrations: array expected";
                            }
                            for (var i = 0; i < message.registrations.length; ++i) {
                                var error = $root.rendezvous.pb.Message.Register.verify(message.registrations[i]);
                                if (error) {
                                    return "registrations." + error;
                                }
                            }
                        }
                        if (message.cookie != null && message.hasOwnProperty("cookie")) {
                            if (
                                !(message.cookie && typeof message.cookie.length === "number"
                                    || $util.isString(message.cookie))
                            ) {
                                return "cookie: buffer expected";
                            }
                        }
                        if (message.status != null && message.hasOwnProperty("status")) {
                            switch (message.status) {
                                default:
                                    return "status: enum value expected";
                                case 0:
                                case 100:
                                case 101:
                                case 102:
                                case 103:
                                case 200:
                                case 300:
                                case 400:
                                    break;
                            }
                        }
                        if (message.statusText != null && message.hasOwnProperty("statusText")) {
                            if (!$util.isString(message.statusText)) {
                                return "statusText: string expected";
                            }
                        }
                        return null;
                    };

                    /**
                     * Creates a DiscoverResponse message from a plain object. Also converts values to their respective internal types.
                     * @function fromObject
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @static
                     * @param {Object.<string,*>} object Plain object
                     * @returns {rendezvous.pb.Message.DiscoverResponse} DiscoverResponse
                     */
                    DiscoverResponse.fromObject = function fromObject(object) {
                        if (object instanceof $root.rendezvous.pb.Message.DiscoverResponse) {
                            return object;
                        }
                        var message = new $root.rendezvous.pb.Message.DiscoverResponse();
                        if (object.registrations) {
                            if (!Array.isArray(object.registrations)) {
                                throw TypeError(
                                    ".rendezvous.pb.Message.DiscoverResponse.registrations: array expected",
                                );
                            }
                            message.registrations = [];
                            for (var i = 0; i < object.registrations.length; ++i) {
                                if (typeof object.registrations[i] !== "object") {
                                    throw TypeError(
                                        ".rendezvous.pb.Message.DiscoverResponse.registrations: object expected",
                                    );
                                }
                                message.registrations[i] = $root.rendezvous.pb.Message.Register.fromObject(
                                    object.registrations[i],
                                );
                            }
                        }
                        if (object.cookie != null) {
                            if (typeof object.cookie === "string") {
                                $util.base64.decode(
                                    object.cookie,
                                    message.cookie = $util.newBuffer($util.base64.length(object.cookie)),
                                    0,
                                );
                            } else if (object.cookie.length) {
                                message.cookie = object.cookie;
                            }
                        }
                        switch (object.status) {
                            case "OK":
                            case 0:
                                message.status = 0;
                                break;
                            case "E_INVALID_NAMESPACE":
                            case 100:
                                message.status = 100;
                                break;
                            case "E_INVALID_SIGNED_PEER_RECORD":
                            case 101:
                                message.status = 101;
                                break;
                            case "E_INVALID_TTL":
                            case 102:
                                message.status = 102;
                                break;
                            case "E_INVALID_COOKIE":
                            case 103:
                                message.status = 103;
                                break;
                            case "E_NOT_AUTHORIZED":
                            case 200:
                                message.status = 200;
                                break;
                            case "E_INTERNAL_ERROR":
                            case 300:
                                message.status = 300;
                                break;
                            case "E_UNAVAILABLE":
                            case 400:
                                message.status = 400;
                                break;
                        }
                        if (object.statusText != null) {
                            message.statusText = String(object.statusText);
                        }
                        return message;
                    };

                    /**
                     * Creates a plain object from a DiscoverResponse message. Also converts values to other types if specified.
                     * @function toObject
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @static
                     * @param {rendezvous.pb.Message.DiscoverResponse} message DiscoverResponse
                     * @param {$protobuf.IConversionOptions} [options] Conversion options
                     * @returns {Object.<string,*>} Plain object
                     */
                    DiscoverResponse.toObject = function toObject(message, options) {
                        if (!options) {
                            options = {};
                        }
                        var object = {};
                        if (options.arrays || options.defaults) {
                            object.registrations = [];
                        }
                        if (options.defaults) {
                            if (options.bytes === String) {
                                object.cookie = "";
                            } else {
                                object.cookie = [];
                                if (options.bytes !== Array) {
                                    object.cookie = $util.newBuffer(object.cookie);
                                }
                            }
                            object.status = options.enums === String ? "OK" : 0;
                            object.statusText = "";
                        }
                        if (message.registrations && message.registrations.length) {
                            object.registrations = [];
                            for (var j = 0; j < message.registrations.length; ++j) {
                                object.registrations[j] = $root.rendezvous.pb.Message.Register.toObject(
                                    message.registrations[j],
                                    options,
                                );
                            }
                        }
                        if (message.cookie != null && message.hasOwnProperty("cookie")) {
                            object.cookie = options.bytes === String
                                ? $util.base64.encode(message.cookie, 0, message.cookie.length)
                                : options.bytes === Array
                                ? Array.prototype.slice.call(message.cookie)
                                : message.cookie;
                        }
                        if (message.status != null && message.hasOwnProperty("status")) {
                            object.status = options.enums === String
                                ? $root.rendezvous.pb.Message.ResponseStatus[message.status]
                                : message.status;
                        }
                        if (message.statusText != null && message.hasOwnProperty("statusText")) {
                            object.statusText = message.statusText;
                        }
                        return object;
                    };

                    /**
                     * Converts this DiscoverResponse to JSON.
                     * @function toJSON
                     * @memberof rendezvous.pb.Message.DiscoverResponse
                     * @instance
                     * @returns {Object.<string,*>} JSON object
                     */
                    DiscoverResponse.prototype.toJSON = function toJSON() {
                        return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                    };

                    return DiscoverResponse;
                })();

                return Message;
            })();

            return pb;
        })();

        return rendezvous;
    })();

    return $root;
});
