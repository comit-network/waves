import * as $protobuf from "protobufjs";
/** Namespace rendezvous. */
export namespace rendezvous {
    /** Namespace pb. */
    namespace pb {
        /** Properties of a Message. */
        interface IMessage {
            /** Message type */
            type?: (rendezvous.pb.Message.MessageType | null);

            /** Message register */
            register?: (rendezvous.pb.Message.IRegister | null);

            /** Message registerResponse */
            registerResponse?: (rendezvous.pb.Message.IRegisterResponse | null);

            /** Message unregister */
            unregister?: (rendezvous.pb.Message.IUnregister | null);

            /** Message discover */
            discover?: (rendezvous.pb.Message.IDiscover | null);

            /** Message discoverResponse */
            discoverResponse?: (rendezvous.pb.Message.IDiscoverResponse | null);
        }

        /** Represents a Message. */
        class Message implements IMessage {
            /**
             * Constructs a new Message.
             * @param [properties] Properties to set
             */
            constructor(properties?: rendezvous.pb.IMessage);

            /** Message type. */
            public type: rendezvous.pb.Message.MessageType;

            /** Message register. */
            public register?: (rendezvous.pb.Message.IRegister | null);

            /** Message registerResponse. */
            public registerResponse?: (rendezvous.pb.Message.IRegisterResponse | null);

            /** Message unregister. */
            public unregister?: (rendezvous.pb.Message.IUnregister | null);

            /** Message discover. */
            public discover?: (rendezvous.pb.Message.IDiscover | null);

            /** Message discoverResponse. */
            public discoverResponse?: (rendezvous.pb.Message.IDiscoverResponse | null);

            /**
             * Creates a new Message instance using the specified properties.
             * @param [properties] Properties to set
             * @returns Message instance
             */
            public static create(properties?: rendezvous.pb.IMessage): rendezvous.pb.Message;

            /**
             * Encodes the specified Message message. Does not implicitly {@link rendezvous.pb.Message.verify|verify} messages.
             * @param message Message message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: rendezvous.pb.IMessage, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified Message message, length delimited. Does not implicitly {@link rendezvous.pb.Message.verify|verify} messages.
             * @param message Message message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: rendezvous.pb.IMessage, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a Message message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns Message
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader | Uint8Array), length?: number): rendezvous.pb.Message;

            /**
             * Decodes a Message message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns Message
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader | Uint8Array)): rendezvous.pb.Message;

            /**
             * Verifies a Message message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string | null);

            /**
             * Creates a Message message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns Message
             */
            public static fromObject(object: { [k: string]: any }): rendezvous.pb.Message;

            /**
             * Creates a plain object from a Message message. Also converts values to other types if specified.
             * @param message Message
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(
                message: rendezvous.pb.Message,
                options?: $protobuf.IConversionOptions,
            ): { [k: string]: any };

            /**
             * Converts this Message to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };
        }

        namespace Message {
            /** MessageType enum. */
            enum MessageType {
                REGISTER = 0,
                REGISTER_RESPONSE = 1,
                UNREGISTER = 2,
                DISCOVER = 3,
                DISCOVER_RESPONSE = 4,
            }

            /** ResponseStatus enum. */
            enum ResponseStatus {
                OK = 0,
                E_INVALID_NAMESPACE = 100,
                E_INVALID_SIGNED_PEER_RECORD = 101,
                E_INVALID_TTL = 102,
                E_INVALID_COOKIE = 103,
                E_NOT_AUTHORIZED = 200,
                E_INTERNAL_ERROR = 300,
                E_UNAVAILABLE = 400,
            }

            /** Properties of a Register. */
            interface IRegister {
                /** Register ns */
                ns?: (string | null);

                /** Register signedPeerRecord */
                signedPeerRecord?: (Uint8Array | null);

                /** Register ttl */
                ttl?: (number | Long | null);
            }

            /** Represents a Register. */
            class Register implements IRegister {
                /**
                 * Constructs a new Register.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: rendezvous.pb.Message.IRegister);

                /** Register ns. */
                public ns: string;

                /** Register signedPeerRecord. */
                public signedPeerRecord: Uint8Array;

                /** Register ttl. */
                public ttl: (number | Long);

                /**
                 * Creates a new Register instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Register instance
                 */
                public static create(properties?: rendezvous.pb.Message.IRegister): rendezvous.pb.Message.Register;

                /**
                 * Encodes the specified Register message. Does not implicitly {@link rendezvous.pb.Message.Register.verify|verify} messages.
                 * @param message Register message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(
                    message: rendezvous.pb.Message.IRegister,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Encodes the specified Register message, length delimited. Does not implicitly {@link rendezvous.pb.Message.Register.verify|verify} messages.
                 * @param message Register message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(
                    message: rendezvous.pb.Message.IRegister,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Decodes a Register message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Register
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(
                    reader: ($protobuf.Reader | Uint8Array),
                    length?: number,
                ): rendezvous.pb.Message.Register;

                /**
                 * Decodes a Register message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Register
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader | Uint8Array)): rendezvous.pb.Message.Register;

                /**
                 * Verifies a Register message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string | null);

                /**
                 * Creates a Register message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Register
                 */
                public static fromObject(object: { [k: string]: any }): rendezvous.pb.Message.Register;

                /**
                 * Creates a plain object from a Register message. Also converts values to other types if specified.
                 * @param message Register
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(
                    message: rendezvous.pb.Message.Register,
                    options?: $protobuf.IConversionOptions,
                ): { [k: string]: any };

                /**
                 * Converts this Register to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };
            }

            /** Properties of a RegisterResponse. */
            interface IRegisterResponse {
                /** RegisterResponse status */
                status?: (rendezvous.pb.Message.ResponseStatus | null);

                /** RegisterResponse statusText */
                statusText?: (string | null);

                /** RegisterResponse ttl */
                ttl?: (number | Long | null);
            }

            /** Represents a RegisterResponse. */
            class RegisterResponse implements IRegisterResponse {
                /**
                 * Constructs a new RegisterResponse.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: rendezvous.pb.Message.IRegisterResponse);

                /** RegisterResponse status. */
                public status: rendezvous.pb.Message.ResponseStatus;

                /** RegisterResponse statusText. */
                public statusText: string;

                /** RegisterResponse ttl. */
                public ttl: (number | Long);

                /**
                 * Creates a new RegisterResponse instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns RegisterResponse instance
                 */
                public static create(
                    properties?: rendezvous.pb.Message.IRegisterResponse,
                ): rendezvous.pb.Message.RegisterResponse;

                /**
                 * Encodes the specified RegisterResponse message. Does not implicitly {@link rendezvous.pb.Message.RegisterResponse.verify|verify} messages.
                 * @param message RegisterResponse message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(
                    message: rendezvous.pb.Message.IRegisterResponse,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Encodes the specified RegisterResponse message, length delimited. Does not implicitly {@link rendezvous.pb.Message.RegisterResponse.verify|verify} messages.
                 * @param message RegisterResponse message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(
                    message: rendezvous.pb.Message.IRegisterResponse,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Decodes a RegisterResponse message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns RegisterResponse
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(
                    reader: ($protobuf.Reader | Uint8Array),
                    length?: number,
                ): rendezvous.pb.Message.RegisterResponse;

                /**
                 * Decodes a RegisterResponse message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns RegisterResponse
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(
                    reader: ($protobuf.Reader | Uint8Array),
                ): rendezvous.pb.Message.RegisterResponse;

                /**
                 * Verifies a RegisterResponse message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string | null);

                /**
                 * Creates a RegisterResponse message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns RegisterResponse
                 */
                public static fromObject(object: { [k: string]: any }): rendezvous.pb.Message.RegisterResponse;

                /**
                 * Creates a plain object from a RegisterResponse message. Also converts values to other types if specified.
                 * @param message RegisterResponse
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(
                    message: rendezvous.pb.Message.RegisterResponse,
                    options?: $protobuf.IConversionOptions,
                ): { [k: string]: any };

                /**
                 * Converts this RegisterResponse to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };
            }

            /** Properties of an Unregister. */
            interface IUnregister {
                /** Unregister ns */
                ns?: (string | null);

                /** Unregister id */
                id?: (Uint8Array | null);
            }

            /** Represents an Unregister. */
            class Unregister implements IUnregister {
                /**
                 * Constructs a new Unregister.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: rendezvous.pb.Message.IUnregister);

                /** Unregister ns. */
                public ns: string;

                /** Unregister id. */
                public id: Uint8Array;

                /**
                 * Creates a new Unregister instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Unregister instance
                 */
                public static create(properties?: rendezvous.pb.Message.IUnregister): rendezvous.pb.Message.Unregister;

                /**
                 * Encodes the specified Unregister message. Does not implicitly {@link rendezvous.pb.Message.Unregister.verify|verify} messages.
                 * @param message Unregister message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(
                    message: rendezvous.pb.Message.IUnregister,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Encodes the specified Unregister message, length delimited. Does not implicitly {@link rendezvous.pb.Message.Unregister.verify|verify} messages.
                 * @param message Unregister message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(
                    message: rendezvous.pb.Message.IUnregister,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Decodes an Unregister message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Unregister
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(
                    reader: ($protobuf.Reader | Uint8Array),
                    length?: number,
                ): rendezvous.pb.Message.Unregister;

                /**
                 * Decodes an Unregister message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Unregister
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(
                    reader: ($protobuf.Reader | Uint8Array),
                ): rendezvous.pb.Message.Unregister;

                /**
                 * Verifies an Unregister message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string | null);

                /**
                 * Creates an Unregister message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Unregister
                 */
                public static fromObject(object: { [k: string]: any }): rendezvous.pb.Message.Unregister;

                /**
                 * Creates a plain object from an Unregister message. Also converts values to other types if specified.
                 * @param message Unregister
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(
                    message: rendezvous.pb.Message.Unregister,
                    options?: $protobuf.IConversionOptions,
                ): { [k: string]: any };

                /**
                 * Converts this Unregister to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };
            }

            /** Properties of a Discover. */
            interface IDiscover {
                /** Discover ns */
                ns?: (string | null);

                /** Discover limit */
                limit?: (number | Long | null);

                /** Discover cookie */
                cookie?: (Uint8Array | null);
            }

            /** Represents a Discover. */
            class Discover implements IDiscover {
                /**
                 * Constructs a new Discover.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: rendezvous.pb.Message.IDiscover);

                /** Discover ns. */
                public ns: string;

                /** Discover limit. */
                public limit: (number | Long);

                /** Discover cookie. */
                public cookie: Uint8Array;

                /**
                 * Creates a new Discover instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Discover instance
                 */
                public static create(properties?: rendezvous.pb.Message.IDiscover): rendezvous.pb.Message.Discover;

                /**
                 * Encodes the specified Discover message. Does not implicitly {@link rendezvous.pb.Message.Discover.verify|verify} messages.
                 * @param message Discover message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(
                    message: rendezvous.pb.Message.IDiscover,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Encodes the specified Discover message, length delimited. Does not implicitly {@link rendezvous.pb.Message.Discover.verify|verify} messages.
                 * @param message Discover message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(
                    message: rendezvous.pb.Message.IDiscover,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Decodes a Discover message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Discover
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(
                    reader: ($protobuf.Reader | Uint8Array),
                    length?: number,
                ): rendezvous.pb.Message.Discover;

                /**
                 * Decodes a Discover message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Discover
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader | Uint8Array)): rendezvous.pb.Message.Discover;

                /**
                 * Verifies a Discover message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string | null);

                /**
                 * Creates a Discover message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Discover
                 */
                public static fromObject(object: { [k: string]: any }): rendezvous.pb.Message.Discover;

                /**
                 * Creates a plain object from a Discover message. Also converts values to other types if specified.
                 * @param message Discover
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(
                    message: rendezvous.pb.Message.Discover,
                    options?: $protobuf.IConversionOptions,
                ): { [k: string]: any };

                /**
                 * Converts this Discover to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };
            }

            /** Properties of a DiscoverResponse. */
            interface IDiscoverResponse {
                /** DiscoverResponse registrations */
                registrations?: (rendezvous.pb.Message.IRegister[] | null);

                /** DiscoverResponse cookie */
                cookie?: (Uint8Array | null);

                /** DiscoverResponse status */
                status?: (rendezvous.pb.Message.ResponseStatus | null);

                /** DiscoverResponse statusText */
                statusText?: (string | null);
            }

            /** Represents a DiscoverResponse. */
            class DiscoverResponse implements IDiscoverResponse {
                /**
                 * Constructs a new DiscoverResponse.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: rendezvous.pb.Message.IDiscoverResponse);

                /** DiscoverResponse registrations. */
                public registrations: rendezvous.pb.Message.IRegister[];

                /** DiscoverResponse cookie. */
                public cookie: Uint8Array;

                /** DiscoverResponse status. */
                public status: rendezvous.pb.Message.ResponseStatus;

                /** DiscoverResponse statusText. */
                public statusText: string;

                /**
                 * Creates a new DiscoverResponse instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns DiscoverResponse instance
                 */
                public static create(
                    properties?: rendezvous.pb.Message.IDiscoverResponse,
                ): rendezvous.pb.Message.DiscoverResponse;

                /**
                 * Encodes the specified DiscoverResponse message. Does not implicitly {@link rendezvous.pb.Message.DiscoverResponse.verify|verify} messages.
                 * @param message DiscoverResponse message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(
                    message: rendezvous.pb.Message.IDiscoverResponse,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Encodes the specified DiscoverResponse message, length delimited. Does not implicitly {@link rendezvous.pb.Message.DiscoverResponse.verify|verify} messages.
                 * @param message DiscoverResponse message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(
                    message: rendezvous.pb.Message.IDiscoverResponse,
                    writer?: $protobuf.Writer,
                ): $protobuf.Writer;

                /**
                 * Decodes a DiscoverResponse message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns DiscoverResponse
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(
                    reader: ($protobuf.Reader | Uint8Array),
                    length?: number,
                ): rendezvous.pb.Message.DiscoverResponse;

                /**
                 * Decodes a DiscoverResponse message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns DiscoverResponse
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(
                    reader: ($protobuf.Reader | Uint8Array),
                ): rendezvous.pb.Message.DiscoverResponse;

                /**
                 * Verifies a DiscoverResponse message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string | null);

                /**
                 * Creates a DiscoverResponse message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns DiscoverResponse
                 */
                public static fromObject(object: { [k: string]: any }): rendezvous.pb.Message.DiscoverResponse;

                /**
                 * Creates a plain object from a DiscoverResponse message. Also converts values to other types if specified.
                 * @param message DiscoverResponse
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(
                    message: rendezvous.pb.Message.DiscoverResponse,
                    options?: $protobuf.IConversionOptions,
                ): { [k: string]: any };

                /**
                 * Converts this DiscoverResponse to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };
            }
        }
    }
}
