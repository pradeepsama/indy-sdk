extern crate libc;

use api::ErrorCode;
use errors::ToErrorCode;
use commands::{Command, CommandExecutor};
use commands::anoncreds::AnoncredsCommand;
use commands::anoncreds::issuer::IssuerCommand;
use commands::anoncreds::prover::ProverCommand;
use commands::anoncreds::verifier::VerifierCommand;
use utils::cstring::CStringUtils;

use self::libc::c_char;

/// Create keys (both primary and revocation) for the given schema and signature type (currently only CL signature type is supported).
/// Store the keys together with signature type and schema in a secure wallet as a claim definition.
/// The claim definition in the wallet is identifying by a returned unique key.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// issuer_did: a DID of the issuer signing claim_def transaction to the Ledger
/// schema_json: schema as a json
/// signature_type: signature type (optional). Currently only 'CL' is supported.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// claim definition json containing information about signature type, schema and issuer's public key.
/// Unique number identifying the public key in the wallet
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn sovrin_issuer_create_and_store_claim_def(command_handle: i32,
                                                       wallet_handle: i32,
                                                       issuer_did: *const c_char,
                                                       schema_json: *const c_char,
                                                       signature_type: *const c_char,
                                                       cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                            claim_def_json: *const c_char,
                                                                            claim_def_wallet_key: i32
                                                       )>) -> ErrorCode {
    check_useful_c_str!(issuer_did, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(schema_json, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(signature_type, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Issuer(IssuerCommand::CreateAndStoreKeys(
            wallet_handle,
            issuer_did,
            schema_json,
            signature_type,
            Box::new(move |result| {
                let (err, claim_def_json, claim_def_wallet_key) = result_to_err_code_2!(result, String::new(), 0);
                let claim_def_json = CStringUtils::string_to_cstring(claim_def_json);

                cb(command_handle, err, claim_def_json.as_ptr(), claim_def_wallet_key)
            })
        ))));

    result_to_err_code!(result)
}

/// Create a new revocation registry for the given claim definition.
/// Stores it in a secure wallet identifying by the returned key.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// issuer_did: a DID of the issuer signing revoc_reg transaction to the Ledger
/// claim_def_seq_no: seq no of a public key transaction in Ledger
/// max_claim_num: maximum number of claims the new registry can process.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Revoc registry json
/// Unique number identifying the revocation registry in the wallet
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn sovrin_issuer_create_and_store_revoc_reg(command_handle: i32,
                                                       wallet_handle: i32,
                                                       issuer_did: *const c_char,
                                                       claim_def_seq_no: i32,
                                                       max_claim_num: i32,
                                                       cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                            revoc_reg_json: *const c_char,
                                                                            revoc_reg_wallet_key: *const c_char
                                                       )>) -> ErrorCode {
    check_useful_c_str!(issuer_did, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Issuer(IssuerCommand::CreateAndStoreRevocation(
            wallet_handle,
            issuer_did,
            claim_def_seq_no,
            max_claim_num,
            Box::new(move |result| {
                let (err, revoc_reg_json, revoc_reg_wallet_key) = result_to_err_code_2!(result, String::new(), String::new());
                let revoc_reg_json = CStringUtils::string_to_cstring(revoc_reg_json);
                let revoc_reg_wallet_key = CStringUtils::string_to_cstring(revoc_reg_wallet_key);

                cb(command_handle, err, revoc_reg_json.as_ptr(), revoc_reg_wallet_key.as_ptr())
            })
        ))));

    result_to_err_code!(result)
}

/// Signs a given claim for the given user by a given key (claim ef).
/// The corresponding claim definition and revocation registry must be already created
/// an stored into the wallet.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// issuer_did: a DID of the issuer signing transactions to the Ledger
/// claim_req_json: a claim request with a blinded secret
///     from the user (returned by prover_create_and_store_claim_req)
/// claim_json: a claim containing attribute values for each of requested attribute names.
///     Example:
///     {
///      "attr1" : ["value1", "value1_as_int"],
///      "attr2" : ["value2", "value2_as_int"]
///     }
/// claim_def_seq_no: seq no of a claim definition transaction in Ledger
/// revoc_reg_seq_no: seq no of a revocation registry transaction in Ledger
/// user_revoc_index: index of a new user in the revocation registry (optional; default one is used if not provided)
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Revocation registry update json with a newly issued claim
/// Claim json containing issued claim, and claim_def_seq_no and revoc_reg_seq_no
/// used for issuance
///     {
///         "claim": string,
///         "claim_def_seq_no": string,
///         "revoc_reg_seq_no", string
///     }
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_issuer_create_claim(command_handle: i32,
                                         wallet_handle: i32,
                                         claim_req_json: *const c_char,
                                         claim_json: *const c_char,
                                         issuer_did: *const c_char,
                                         claim_def_seq_no: i32,
                                         revoc_reg_seq_no: i32,
                                         user_revoc_index: i32,
                                         cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                              revoc_reg_update_json: *const c_char,
                                                              xclaim_json: *const c_char
                                         )>) -> ErrorCode {
    check_useful_c_str!(claim_req_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(claim_json, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(issuer_did, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Issuer(IssuerCommand::CreateClaim(
            wallet_handle,
            claim_req_json,
            claim_json,
            issuer_did,
            claim_def_seq_no,
            revoc_reg_seq_no,
            user_revoc_index,
            Box::new(move |result| {
                let (err, revoc_reg_update_json, xclaim_json) = result_to_err_code_2!(result, String::new(), String::new());
                let revoc_reg_update_json = CStringUtils::string_to_cstring(revoc_reg_update_json);
                let xclaim_json = CStringUtils::string_to_cstring(xclaim_json);
                cb(command_handle, err, revoc_reg_update_json.as_ptr(), xclaim_json.as_ptr())
            })
        ))));

    result_to_err_code!(result)
}

/// Revokes a user identified by a revoc_id in a given revoc-registry.
/// The corresponding claim definition and revocation registry must be already
/// created an stored into the wallet.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// issuer_did: a DID of the issuer signing transactions to the Ledger
/// claim_def_seq_no: seq no of a claim definition transaction in Ledger
/// revoc_reg_seq_no: seq no of a revocation registry transaction in Ledger
/// user_revoc_index: index of the user in the revocation registry
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Revocation registry update json with a revoked claim
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_issuer_revoke_claim(command_handle: i32,
                                         wallet_handle: i32,
                                         issuer_did: *const c_char,
                                         claim_def_seq_no: i32,
                                         revoc_reg_seq_no: i32,
                                         user_revoc_index: i32,
                                         cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                              revoc_reg_update_json: *const c_char,
                                         )>) -> ErrorCode {
    check_useful_c_str!(issuer_did, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Issuer(IssuerCommand::RevokeClaim(
            wallet_handle,
            issuer_did,
            claim_def_seq_no,
            revoc_reg_seq_no,
            user_revoc_index,
            Box::new(move |result| {
                let (err, revoc_reg_update_json) = result_to_err_code_1!(result, String::new());
                let revoc_reg_update_json = CStringUtils::string_to_cstring(revoc_reg_update_json);

                cb(command_handle, err, revoc_reg_update_json.as_ptr())
            })
        ))));

    result_to_err_code!(result)
}

/// Stores a claim offer from the given issuer in a secure storage.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// claim_offer_json: claim offer as a json containing information about the issuer and a claim:
///        {
///            "issuer_did": string,
///            "claim_def_seq_no": string
///        }
///
/// #Returns
/// None.
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_prover_store_claim_offer(command_handle: i32,
                                              wallet_handle: i32,
                                              claim_offer_json: *const c_char,
                                              cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode
                                              )>) -> ErrorCode {
    check_useful_c_str!(claim_offer_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::StoreClaimOffer(
            wallet_handle,
            claim_offer_json,
            Box::new(move |result| {
                let err = result_to_err_code!(result);
                cb(command_handle, err)
            })
        ))));

    result_to_err_code!(result)
}

/// Gets all stored claim offers (see prover_store_claim_offer).
/// A filter can be specified to get claim offers for specific Issuer only.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// filter_json: optional filter to get claim offers for specific Issuer only
///        {
///            "issuer_did": string
///        }
///
/// #Returns
/// A json with a ist of claim offers for the filter.
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_prover_get_claim_offers(command_handle: i32,
                                             wallet_handle: i32,
                                             filter_json: *const c_char,
                                             cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                  claim_offers_json: *const c_char
                                             )>) -> ErrorCode {
    check_useful_c_str!(filter_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::GetClaimOffers(
            wallet_handle,
            filter_json,
            Box::new(move |result| {
                let (err, claim_offers_json) = result_to_err_code_1!(result, String::new());
                let claim_offers_json = CStringUtils::string_to_cstring(claim_offers_json);

                cb(command_handle, err, claim_offers_json.as_ptr())
            })
        ))));

    result_to_err_code!(result)
}


/// Creates a master secret with a given name and stores it in the wallet.
/// The name must be unique.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// master_secret_name: a new master secret name
///
/// #Returns
/// None.
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_prover_create_master_secret(command_handle: i32,
                                                 wallet_handle: i32,
                                                 master_secret_name: *const c_char,
                                                 cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode
                                                 )>) -> ErrorCode {
    check_useful_c_str!(master_secret_name, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::CreateMasterSecret(
            wallet_handle,
            master_secret_name,
            Box::new(move |result| {
                let err = result_to_err_code!(result);
                cb(command_handle, err)
            })
        ))));

    result_to_err_code!(result)
}

/// Creates a clam request json for the given claim offer and stores it in a secure wallet.
/// The claim offer contains the information about Issuer (DID, claim_def_seq_no),
/// and the schema (schema_seq_no).
/// The method gets public key and schema from the ledger, stores them in a wallet,
/// and creates a blinded master secret for a master secret identified by a provided name.
/// The master secret identified by the name must be already stored in the secure wallet (see prover_create_master_secret)
/// The blinded master secret is a part of the claim request.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// claim_offer_json: claim offer as a json containing information about the issuer and a claim:
///        {
///            "issuer_did": string,
///            "claim_def_seq_no": string
///        }
/// claim_def_json: claim definition json associated with a schema_seq_no in the claim_offer
/// master_secret_name: the name of the master secret stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Claim request json.
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_prover_create_and_store_claim_req(command_handle: i32,
                                                       wallet_handle: i32,
                                                       claim_offer_json: *const c_char,
                                                       claim_def_json: *const c_char,
                                                       master_secret_name: *const c_char,
                                                       cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                            claim_req_json: *const c_char
                                                       )>) -> ErrorCode {
    check_useful_c_str!(claim_offer_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(claim_def_json, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(master_secret_name, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::CreateAndStoreClaimRequest(
            wallet_handle,
            claim_offer_json,
            claim_def_json,
            master_secret_name,
            Box::new(move |result| {
                let (err, claim_req_json) = result_to_err_code_1!(result, String::new());
                let claim_req_json = CStringUtils::string_to_cstring(claim_req_json);

                cb(command_handle, err, claim_req_json.as_ptr())
            })
        ))));

    result_to_err_code!(result)
}

/// Updates the claim by a master secret and stores in a secure wallet.
/// The claim contains the information about
/// claim_def_seq_no revoc_reg_seq_no (see issuer_create_claim).
/// Seq_no is a sequence number of the corresponding transaction in the ledger.
/// The method loads a blinded secret for this key from the wallet,
/// updates the claim and stores it in a wallet.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// claims_json: claim json:
///     {
///         "claim": string,
///         "claim_def_seq_no", string,
///         "revoc_reg_seq_no", string
///     }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// None
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_prover_store_claim(command_handle: i32,
                                        wallet_handle: i32,
                                        claims_json: *const c_char,
                                        cb: Option<extern fn(
                                            xcommand_handle: i32, err: ErrorCode
                                        )>) -> ErrorCode {
    check_useful_c_str!(claims_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::StoreClaim(
            wallet_handle,
            claims_json,
            Box::new(move |result| {
                let err = result_to_err_code!(result);
                cb(command_handle, err)
            })
        ))));

    result_to_err_code!(result)
}


/// Gets human readable claims according to the filter.
/// If filter is NULL, then all claims are returned.
/// Claims can be filtered by Issuer and/or Schema, or by proof_request.
/// If filtered by pool_request, than all claims matching the given proof_request are returned
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// filter_json: filter for claims
///     {
///         "issuer_did": string,
///         "schema_seq_no": string,
///         "claim_def_seq_no": string,
///         "proof_request": string
///     }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// claims json
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_prover_get_claims(command_handle: i32,
                                        wallet_handle: i32,
                                        filter_json: *const c_char,
                                        cb: Option<extern fn(
                                            xcommand_handle: i32, err: ErrorCode,
                                            claims_json: *const c_char
                                        )>) -> ErrorCode {
    check_useful_c_str!(filter_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::GetClaims(
            wallet_handle,
            filter_json,
            Box::new(move |result| {
                let (err, claims_json) = result_to_err_code_1!(result, String::new());
                let claims_json = CStringUtils::string_to_cstring(claims_json);

                cb(command_handle, err, claims_json.as_ptr())
            })
        ))));

    result_to_err_code!(result)
}

/// Gets human readable claims matching the given proof request.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// proof_request_json: proof request json
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// json with claims for the given pool request.
/// Claim consists of human-readable attributes (key-value map), schema_seq_no, claim_def_seq_no and revoc_reg_seq_no.
///     {
///         "requested_attr1_id": [claim1, claim2],
///         "requested_attr2_id": [],
///         "requested_attr3_id": [claim3],
///     }
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_prover_get_claims_for_pool_req(command_handle: i32,
                                        wallet_handle: i32,
                                        proof_request_json: *const c_char,
                                        cb: Option<extern fn(
                                            xcommand_handle: i32, err: ErrorCode,
                                            claims_json: *const c_char
                                        )>) -> ErrorCode {
    check_useful_c_str!(proof_request_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::GetClaimsForProofReq(
            wallet_handle,
            proof_request_json,
            Box::new(move |result| {
                let (err, claims_json) = result_to_err_code_1!(result, String::new());
                let claims_json = CStringUtils::string_to_cstring(claims_json);

                cb(command_handle, err, claims_json.as_ptr())
            })
        ))));

    result_to_err_code!(result)
}

/// Creates a proof according to the given proof request
/// Either a corresponding claim with revealed attributes or self-attested attribute must be provided
/// for each requested attribute (see sovrin_prover_get_claims_for_pool_req).
/// A proof request may request multiple claims from different schemas and different issuers.
/// All required schemas, public keys and revocation registries must be provided.
/// The proof request also contains nonce.
/// The proof contains either proof or self-attested attribute value for each requested attribute.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// proof_req_json: proof request json as come from the verifier
/// requested_claims_json: either a claim or self-attested attribute for each requested attribute
///     {
///         "requested_attr1_id": [claim1_seq_no_in_wallet, revealed attribute1],
///         "requested_attr2_id": [self_attested_attribute],
///         "requested_attr3_id": [claim2_seq_no_in_wallet, revealed attribute2]
///         "requested_attr4_id": [claim2_seq_no_in_wallet, revealed attribute3]
///     }
/// schemas_jsons: all schema jsons participating in the proof request
/// claim_def_jsons: all claim definition jsons participating in the proof request
/// revoc_regs_jsons: all revocation registry jsons participating in the proof request
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Proof json
///  Each proof consists of a proof and corresponding schema_seq_no, claim_def_seq_no and revoc_reg_seq_no.
///     {
///         "requested_attr1_id": [proof1],
///         "requested_attr2_id": [self_attested_attribute],
///         "requested_attr3_id": [proof2]
///         "requested_attr4_id": [proof3]
///     }
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_prover_create_proof(command_handle: i32,
                                         wallet_handle: i32,
                                         proof_req_json: *const c_char,
                                         requested_claims_json: *const c_char,
                                         schemas_json: *const c_char,
                                         claim_defs_json: *const c_char,
                                         revoc_regs_json: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                              proof_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(proof_req_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(requested_claims_json, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(schemas_json, ErrorCode::CommonInvalidParam5);
    check_useful_c_str!(claim_defs_json, ErrorCode::CommonInvalidParam6);
    check_useful_c_str!(revoc_regs_json, ErrorCode::CommonInvalidParam7);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::CreateProof(
            wallet_handle,
            proof_req_json,
            requested_claims_json,
            schemas_json,
            claim_defs_json,
            revoc_regs_json,
            Box::new(move |result| {
                let (err, proof_json) = result_to_err_code_1!(result, String::new());
                let proof_json = CStringUtils::string_to_cstring(proof_json);

                cb(command_handle, err, proof_json.as_ptr())
            })
        ))));

    result_to_err_code!(result)
}

/// Verifies a proof (of multiple claim).
/// All required schemas, public keys and revocation registries must be provided.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// proof_request_json: initial proof request as sent by the verifier
/// proof_json: proof json
///  Each proof consists of a proof and corresponding schema_seq_no, claim_def_seq_no and revoc_reg_seq_no.
///     {
///         "requested_attr1_id": [proof1],
///         "requested_attr2_id": [self_attested_attribute],
///         "requested_attr3_id": [proof2]
///         "requested_attr4_id": [proof3]
///     }
/// schemas_jsons: all schema jsons participating in the proof
/// claim_defs_jsons: all claim definition jsons participating in the proof
/// revoc_regs_jsons: all revocation registry jsons participating in the proof
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// valid: true - if signature is valid, false - otherwise
///
/// #Errors
/// Annoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn sovrin_verifier_verify_proof(command_handle: i32,
                                           wallet_handle: i32,
                                           proof_request_json: *const c_char,
                                           proof_json: *const c_char,
                                           schemas_json: *const c_char,
                                           claim_defs_jsons: *const c_char,
                                           revoc_regs_json: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                valid: bool)>) -> ErrorCode {
    check_useful_c_str!(proof_request_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(proof_json, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(schemas_json, ErrorCode::CommonInvalidParam5);
    check_useful_c_str!(claim_defs_jsons, ErrorCode::CommonInvalidParam6);
    check_useful_c_str!(revoc_regs_json, ErrorCode::CommonInvalidParam7);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Verifier(VerifierCommand::VerifyProof(
            wallet_handle,
            proof_request_json,
            proof_json,
            schemas_json,
            claim_defs_jsons,
            revoc_regs_json,
            Box::new(move |result| {
                let (err, valid) = result_to_err_code_1!(result, false);
                cb(command_handle, err, valid)
            })
        ))));

    result_to_err_code!(result)
}