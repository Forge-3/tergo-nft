DEFAULT=$(dfx identity get-principal)

dfx deploy tergo_nft --argument "(record{
  minting_account= opt record {
    owner = principal \"$DEFAULT\";
    subaccount = opt blob \"\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\";
  };
  icrc7_supply_cap= null;
  icrc7_description= opt \"Tergo NFT Collection\";
  tx_window= null;
  permitted_drift= null;
  icrc7_max_take_value= opt 100;
  icrc7_max_memo_size= opt 1000;
  icrc7_symbol= \"TNC\";
  icrc7_max_update_batch_size= opt 100;
  icrc7_max_query_batch_size= opt 100;
  icrc7_atomic_batch_transfers= null;
  icrc7_default_take_value= opt 100;
  icrc7_logo= null;
  icrc7_name= \"Tergo NFT\";
  approval_init= null;
  archive_init= opt record {
    maxRecordsToArchive= 2;
    archiveIndexType= variant {Stable};
    maxArchivePages= 3;
    settleToRecords= 2;
    archiveCycles= 5000000000000;
    maxActiveRecords= 4;
    maxRecordsInArchiveInstance= 4;
    archiveControllers= null
  };
})"

dfx deploy tergo_nft_archive --argument "(record {
  max_records= 2;
  index_type= variant {Stable};
  first_index= 0;
  max_pages= 3;
})"